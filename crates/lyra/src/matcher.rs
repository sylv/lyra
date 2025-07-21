use crate::entities::media::{self, MediaType};
use crate::entities::{file, media_connection};
use crate::tmdb::{TMDB_IMAGE_BASE_URL, TMDBClient};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub async fn start_matcher(pool: DatabaseConnection) -> anyhow::Result<()> {
    info!("Starting matcher");
    let tmdb_client = TMDBClient::new();
    loop {
        let file = file::Entity::find()
            .filter(file::Column::PendingAutoMatch.eq(1))
            .order_by_asc(file::Column::Id)
            .one(&pool)
            .await?;

        match file {
            Some(file) => {
                info!("processing unmatched file '{}'", file.key);
                match process_file(&pool, &file, &tmdb_client).await {
                    Ok(_) => {
                        file::Entity::update(file::ActiveModel {
                            id: Set(file.id),
                            pending_auto_match: Set(0),
                            ..Default::default()
                        })
                        .exec(&pool)
                        .await?;
                    }
                    Err(e) => {
                        warn!("failed to process file '{}': {}", file.key, e);
                        sleep(Duration::from_secs(30)).await;
                    }
                }
            }
            None => {
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_file(
    pool: &DatabaseConnection,
    file: &file::Model,
    tmdb_client: &TMDBClient,
) -> anyhow::Result<()> {
    let metadata = torrent_name_parser::Metadata::from(&file.key)?;

    if metadata.is_show() {
        process_show(pool, &file.key, file.id, &metadata, tmdb_client).await?;
    } else {
        process_movie(pool, &file.key, file.id, &metadata, tmdb_client).await?;
    }

    Ok(())
}

async fn process_movie(
    pool: &DatabaseConnection,
    file_key: &str,
    file_id: i64,
    metadata: &torrent_name_parser::Metadata,
    tmdb_client: &TMDBClient,
) -> anyhow::Result<()> {
    let search_results = tmdb_client
        .search_movie(
            metadata.title(),
            metadata.year().map(|y| y.to_string()).as_deref(),
        )
        .await?;

    let Some(movie_result) = search_results.results.first() else {
        warn!("no movie results found for '{}'", metadata.title());
        return Ok(());
    };

    tracing::info!(
        "matched file '{}' to movie '{} ({})'",
        file_key,
        movie_result.title,
        movie_result.id
    );

    let movie_details = tmdb_client.get_movie_details(movie_result.id).await?;

    let release_date = movie_details
        .release_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

    let poster_url = movie_details
        .poster_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));
    let background_url = movie_details
        .backdrop_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));

    let media = media::ActiveModel {
        media_type: Set(MediaType::Movie),
        tmdb_parent_id: Set(movie_details.id),
        tmdb_item_id: Set(movie_details.id),
        name: Set(movie_details.title),
        description: Set(movie_details.overview),
        rating: Set(movie_details.vote_average),
        start_date: Set(release_date),
        runtime_minutes: Set(movie_details.runtime),
        poster_url: Set(poster_url),
        background_url: Set(background_url),
        ..Default::default()
    };

    // let media = media.insert(pool).await?;
    let media = media::Entity::insert(media)
        .on_conflict(
            OnConflict::columns([media::Column::TmdbParentId, media::Column::TmdbItemId])
                .update_columns([
                    media::Column::Name,
                    media::Column::Description,
                    media::Column::Rating,
                    media::Column::StartDate,
                    media::Column::RuntimeMinutes,
                    media::Column::PosterUrl,
                    media::Column::BackgroundUrl,
                ])
                .to_owned(),
        )
        .exec_with_returning(pool)
        .await?;

    let connection = media_connection::ActiveModel {
        media_id: Set(media.id),
        file_id: Set(file_id),
    };
    connection.insert(pool).await?;

    Ok(())
}

async fn process_show(
    pool: &DatabaseConnection,
    file_key: &str,
    file_id: i64,
    metadata: &torrent_name_parser::Metadata,
    tmdb_client: &TMDBClient,
) -> anyhow::Result<()> {
    let search_results = tmdb_client
        .search_tv(
            metadata.title(),
            metadata.year().map(|y| y.to_string()).as_deref(),
        )
        .await?;

    let Some(season_number) = metadata.season() else {
        return Ok(());
    };
    let season_number_i64 = season_number as i64;

    // Find a show result that has the required season number
    let mut show_result = None;
    let mut show_details = None;

    for result in &search_results.results {
        let details = tmdb_client.get_tv_show_details(result.id).await?;

        // Check if this show has the required season number
        if details
            .seasons
            .iter()
            .any(|s| s.season_number == season_number_i64)
        {
            show_result = Some(result);
            show_details = Some(details);
            break;
        } else {
            tracing::debug!(
                "skipping show '{}' ({}) - only has {} seasons, but file requires season {}",
                result.name,
                result.id,
                details.seasons.len(),
                season_number_i64
            );
        }
    }

    let (show_result, show_details) = match (show_result, show_details) {
        (Some(result), Some(details)) => (result, details),
        _ => {
            warn!(
                "no show results found for '{}' with season {}",
                metadata.title(),
                season_number_i64
            );
            return Ok(());
        }
    };

    tracing::info!(
        "matched file '{}' to show '{} ({})' with season {}",
        file_key,
        show_result.name,
        show_result.id,
        season_number_i64
    );

    let start_date = show_details
        .first_air_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

    let end_date = show_details
        .last_air_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

    // // Create or update the show
    // let mut show = UpsertMedia::new(show_details.name, MediaType::Show);
    // show.description = show_details.overview;
    // show.rating = show_details.vote_average;
    // show.start_date = start_date;
    // show.tmdb_parent_id = Some(show_details.id);
    // show.tmdb_item_id = Some(show_details.id);

    let poster_url = show_details
        .poster_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));
    let background_url = show_details
        .backdrop_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));

    let mut show = media::ActiveModel {
        media_type: Set(MediaType::Show),
        tmdb_parent_id: Set(show_details.id),
        tmdb_item_id: Set(show_details.id),
        name: Set(show_details.name),
        description: Set(show_details.overview),
        rating: Set(show_details.vote_average),
        start_date: Set(start_date),
        poster_url: Set(poster_url),
        background_url: Set(background_url),
        ..Default::default()
    };

    // only set end_date if the show has ended
    if show_details.in_production {
        show.end_date = Set(None);
    } else {
        show.end_date = Set(end_date);
    }

    let show = media::Entity::insert(show)
        .on_conflict(
            OnConflict::columns([media::Column::TmdbParentId, media::Column::TmdbItemId])
                .update_columns([
                    media::Column::Name,
                    media::Column::Description,
                    media::Column::Rating,
                    media::Column::StartDate,
                    media::Column::RuntimeMinutes,
                    media::Column::PosterUrl,
                    media::Column::BackgroundUrl,
                ])
                .to_owned(),
        )
        .exec_with_returning(pool)
        .await?;

    let season_details = tmdb_client
        .get_tv_season_details(show_details.id, season_number_i64)
        .await?;

    // insert all episodes of this season, not just the ones we have files for
    for episode_details in season_details.episodes {
        // let mut episode = UpsertMedia::new(episode_details.name.clone(), MediaType::Episode);
        // let release_date = episode_details
        //     .air_date
        //     .as_ref()
        //     .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        //     .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

        // episode.start_date = release_date;

        // episode.description = episode_details.overview.clone();
        // episode.parent_id = Some(show.id);
        // episode.season_number = Some(season_number_i64);
        // episode.episode_number = Some(episode_details.episode_number);
        // episode.runtime_minutes = episode_details.runtime;
        // episode.rating = episode_details.vote_average;
        // episode.tmdb_parent_id = Some(show_details.id);
        // episode.tmdb_item_id = Some(episode_details.id);
        // episode.thumbnail_url = episode_details
        //     .still_path
        //     .as_ref()
        //     .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));

        let release_date = episode_details
            .air_date
            .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

        let thumbnail_url = episode_details
            .still_path
            .as_ref()
            .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));

        let episode = media::ActiveModel {
            media_type: Set(MediaType::Episode),
            tmdb_parent_id: Set(show_details.id),
            tmdb_item_id: Set(episode_details.id),
            name: Set(episode_details.name),
            description: Set(episode_details.overview),
            rating: Set(episode_details.vote_average),
            start_date: Set(release_date),
            thumbnail_url: Set(thumbnail_url),
            parent_id: Set(Some(show.id)),
            season_number: Set(Some(season_number_i64)),
            episode_number: Set(Some(episode_details.episode_number)),
            runtime_minutes: Set(episode_details.runtime),
            ..Default::default()
        };

        let episode = media::Entity::insert(episode)
            .on_conflict(
                OnConflict::columns([media::Column::TmdbParentId, media::Column::TmdbItemId])
                    .update_columns([
                        media::Column::Name,
                        media::Column::Description,
                        media::Column::Rating,
                        media::Column::StartDate,
                        media::Column::RuntimeMinutes,
                        media::Column::PosterUrl,
                        media::Column::BackgroundUrl,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(pool)
            .await?;

        // Connect the file to the episode if this episode is in the file
        if metadata
            .episodes()
            .contains(&(episode_details.episode_number as i32))
        {
            let connection = media_connection::ActiveModel {
                media_id: Set(episode.id),
                file_id: Set(file_id),
            };

            connection.insert(pool).await?;
        }
    }

    Ok(())
}
