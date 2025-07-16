use crate::entities::media::{MediaType, UpsertMedia};
use crate::entities::media_connection::MediaConnection;
use crate::tmdb::{TMDB_IMAGE_BASE_URL, TMDBClient};
use sqlx::{SqlitePool, types::chrono};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

pub async fn start_matcher(pool: SqlitePool) -> anyhow::Result<()> {
    info!("Starting matcher");
    let tmdb_client = TMDBClient::new();
    loop {
        let file = sqlx::query_as::<_, File>(
            r#"
            SELECT id, key
            FROM file
            WHERE pending_auto_match = 1
            LIMIT 1
            "#,
        )
        .fetch_optional(&pool)
        .await?;

        match file {
            Some(file) => {
                info!("processing unmatched file '{}'", file.key);
                match process_file(&pool, &file, &tmdb_client).await {
                    Ok(_) => {
                        sqlx::query!(
                            "UPDATE file SET pending_auto_match = 0 WHERE id = ?",
                            file.id
                        )
                        .execute(&pool)
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

#[derive(sqlx::FromRow)]
struct File {
    id: i64,
    key: String,
}

async fn process_file(
    pool: &SqlitePool,
    file: &File,
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
    pool: &SqlitePool,
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

    let mut upsert_media =
        UpsertMedia::new(movie_details.title, MediaType::Movie, movie_details.id);
    upsert_media.description = movie_details.overview;
    upsert_media.rating = movie_details.vote_average;
    upsert_media.release_date = release_date;
    upsert_media.runtime_minutes = movie_details.runtime;
    upsert_media.poster_url = movie_details
        .poster_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));
    upsert_media.background_url = movie_details
        .backdrop_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));

    let media = upsert_media.upsert(pool).await?;

    MediaConnection::create(pool, media.id, file_id).await?;

    Ok(())
}

async fn process_show(
    pool: &SqlitePool,
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

    let release_date = show_details
        .first_air_date
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp());

    // Create or update the show
    let mut show = UpsertMedia::new(show_details.name, MediaType::Show, show_details.id);
    show.description = show_details.overview;
    show.rating = show_details.vote_average;
    show.release_date = release_date;
    show.poster_url = show_details
        .poster_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));
    show.background_url = show_details
        .backdrop_path
        .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));
    let show = show.upsert(pool).await?;

    let season_details = tmdb_client
        .get_tv_season_details(show_details.id, season_number_i64)
        .await?;

    // Create or update the season
    let mut season = UpsertMedia::new(season_details.name, MediaType::Season, show_details.id);
    season.description = season_details.overview;
    season.parent_id = Some(show.id);
    season.season_number = Some(season_number_i64);

    let season = season.upsert(pool).await?;

    // insert ALL episodes of this season, not just the ones we have files for
    for episode_details in &season_details.episodes {
        let mut episode = UpsertMedia::new(
            episode_details.name.clone(),
            MediaType::Episode,
            show_details.id,
        );
        episode.description = episode_details.overview.clone();
        episode.parent_id = Some(season.id);
        episode.season_number = Some(season_number_i64);
        episode.episode_number = Some(episode_details.episode_number);
        episode.runtime_minutes = episode_details.runtime;
        episode.rating = episode_details.vote_average;
        episode.tmdb_item_id = Some(episode_details.id);
        episode.thumbnail_url = episode_details
            .still_path
            .as_ref()
            .map(|path| format!("{}{}", TMDB_IMAGE_BASE_URL, path));

        let episode = episode.upsert(pool).await?;

        // Connect the file to the episode if this episode is in the file
        if metadata
            .episodes()
            .contains(&(episode_details.episode_number as i32))
        {
            MediaConnection::create(pool, episode.id, file_id).await?;
        }
    }

    Ok(())
}
