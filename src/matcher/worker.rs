use crate::entities::media::{self, MediaType};
use crate::entities::{file, media_connection};
use crate::matcher::matcher::{MatchResult, match_file_to_metadata};
use crate::tmdb::{MovieDetails, TMDB_IMAGE_BASE_URL, TMDBClient, TvShowDetails};
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
    let matched = match_file_to_metadata(tmdb_client, &file.key).await?;

    match matched {
        Some(MatchResult::Movie(movie)) => {
            tracing::debug!(
                "matched '{}' to movie '{}' ({})",
                file.key,
                movie.title,
                movie.id
            );
            process_movie(pool, file.id, movie).await?;
        }
        Some(MatchResult::Series { show, parsed }) => {
            tracing::debug!(
                "matched '{}' to series '{}' ({})",
                file.key,
                show.name,
                show.id
            );
            let season = parsed
                .season_number
                .expect("series should have a season number");
            let episodes = parsed.episodes;
            process_show(pool, file.id, show, season, episodes, tmdb_client).await?;
        }
        None => {
            warn!("no match found for '{}'", file.key);
        }
    }

    Ok(())
}

async fn process_movie(
    pool: &DatabaseConnection,
    file_id: i64,
    movie_details: MovieDetails,
) -> anyhow::Result<()> {
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

    let imdb_id = movie_details
        .external_ids
        .as_ref()
        .and_then(|ids| ids.imdb_id.as_ref())
        .map(|id| id.to_string());

    let media = media::ActiveModel {
        media_type: Set(MediaType::Movie),
        tmdb_parent_id: Set(movie_details.id),
        tmdb_item_id: Set(movie_details.id),
        imdb_parent_id: Set(imdb_id),
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
    file_id: i64,
    show_details: TvShowDetails,
    season_number: i32,
    episodes: Vec<i32>,
    tmdb_client: &TMDBClient,
) -> anyhow::Result<()> {
    let start_date = show_details
        .first_air_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

    let end_date = show_details
        .last_air_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

    let poster_url = show_details
        .poster_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));
    let background_url = show_details
        .backdrop_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));

    let imdb_id = show_details
        .external_ids
        .as_ref()
        .and_then(|ids| ids.imdb_id.as_ref())
        .map(|id| id.to_string());

    let mut show = media::ActiveModel {
        media_type: Set(MediaType::Show),
        tmdb_parent_id: Set(show_details.id),
        tmdb_item_id: Set(show_details.id),
        imdb_parent_id: Set(imdb_id),
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

    let season_details = match tmdb_client
        .get_tv_season_details(show_details.id, season_number as i64)
        .await
    {
        Ok(details) => details,
        Err(e) => {
            // todo: this is to handle tmdb season/episode misalignment (eg, imdb lists dandadan as having 2 seasons but tmdb defaults to showing 1)
            // we should handle 404 explicitly, and in the future still import it somehow
            warn!(
                "failed to get season {} details for show {}: {}",
                season_number, show_details.id, e
            );
            return Ok(());
        }
    };

    // insert all episodes of this season, not just the ones we have files for
    for episode_details in season_details.episodes {
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
            season_number: Set(Some(season_number as i64)),
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
        if episodes.contains(&(episode_details.episode_number as i32)) {
            let connection = media_connection::ActiveModel {
                media_id: Set(episode.id),
                file_id: Set(file_id),
            };

            media_connection::Entity::insert(connection)
                .on_conflict(
                    OnConflict::columns([
                        media_connection::Column::MediaId,
                        media_connection::Column::FileId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec_without_returning(pool)
                .await?;
        }
    }

    Ok(())
}
