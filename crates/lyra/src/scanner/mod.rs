pub mod derivation;
pub mod local;

use crate::config::get_config;
use crate::entities::{
    files, item_files, item_metadata, items, libraries, metadata_source::MetadataSource,
    root_metadata, roots, season_metadata, seasons,
};
use crate::scanner::derivation::derive_library_media;
use crate::scanner::local::{
    insert_local_item_metadata, insert_local_root_metadata, insert_local_season_metadata,
};
use lyra_parser::{ParsedFile, parse_files};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder,
    TransactionTrait,
};
use std::path::Path as StdPath;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{Duration, sleep};

const MIN_FILE_SIZE_MB: u64 = 25 * 1024 * 1024;
const PARSE_BATCH_SIZE: usize = 100;
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "webm"];

pub async fn start_scanner(
    pool: DatabaseConnection,
    wake_signal: Arc<Notify>,
) -> anyhow::Result<()> {
    loop {
        let config = get_config();
        let scan_ago_filter = chrono::Utc::now().timestamp() - config.library_scan_interval;
        let to_scan = libraries::Entity::find()
            .filter(
                libraries::Column::LastScannedAt
                    .lt(scan_ago_filter)
                    .or(libraries::Column::LastScannedAt.is_null()),
            )
            .one(&pool)
            .await?;

        if let Some(library) = to_scan {
            scan_library(&pool, &library, &wake_signal).await?;
        } else {
            sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn scan_library(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    wake_signal: &Arc<Notify>,
) -> anyhow::Result<()> {
    let scan_start_time = chrono::Utc::now().timestamp();
    let library_path = PathBuf::from(&library.path);

    tracing::info!(
        "Scanning directory: {} for library: {}",
        library_path.display(),
        library.name
    );

    scan_directory(pool, library, &library_path, &library_path, scan_start_time).await?;

    files::Entity::update_many()
        .set(files::ActiveModel {
            unavailable_at: Set(Some(scan_start_time)),
            ..Default::default()
        })
        .filter(files::Column::LibraryId.eq(library.id))
        .filter(files::Column::ScannedAt.lt(scan_start_time))
        .filter(files::Column::UnavailableAt.is_null())
        .exec(pool)
        .await?;

    rebuild_library_media(pool, library, &library_path).await?;

    libraries::Entity::update(libraries::ActiveModel {
        id: Set(library.id),
        last_scanned_at: Set(Some(chrono::Utc::now().timestamp())),
        ..Default::default()
    })
    .exec(pool)
    .await?;

    wake_signal.notify_waiters();
    tracing::info!("Scan completed for library '{}'", library.name);
    Ok(())
}

#[async_recursion::async_recursion]
async fn scan_directory(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    root_dir: &StdPath,
    current_dir: &StdPath,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    let mut entries = tokio::fs::read_dir(current_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(pool, library, root_dir, &path, scan_start_time).await?;
        } else if path.is_file() {
            scan_file(pool, library, &path, root_dir, scan_start_time).await?;
        }
    }

    Ok(())
}

async fn scan_file(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    path: &PathBuf,
    root_dir: &StdPath,
    scan_start_time: i64,
) -> anyhow::Result<()> {
    let Some(extension) = path.extension().and_then(|e| e.to_str()) else {
        return Ok(());
    };

    if !VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
        return Ok(());
    }

    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            tracing::error!(
                "error getting metadata for file {}, ignoring",
                path.display()
            );
            return Ok(());
        }
    };

    if metadata.len() < MIN_FILE_SIZE_MB {
        return Ok(());
    }

    let relative_path = path
        .strip_prefix(root_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    files::Entity::insert(files::ActiveModel {
        library_id: Set(library.id),
        relative_path: Set(relative_path.clone()),
        size_bytes: Set(metadata.len() as i64),
        audio_fingerprint: Set(Vec::new()),
        segments_json: Set(Vec::new()),
        keyframes_json: Set(Vec::new()),
        scanned_at: Set(Some(scan_start_time)),
        unavailable_at: Set(None),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::columns([files::Column::LibraryId, files::Column::RelativePath])
            .update_columns([
                files::Column::SizeBytes,
                files::Column::ScannedAt,
                files::Column::UnavailableAt,
            ])
            .to_owned(),
    )
    .exec(pool)
    .await?;

    Ok(())
}

async fn rebuild_library_media(
    pool: &DatabaseConnection,
    library: &libraries::Model,
    library_root: &StdPath,
) -> anyhow::Result<()> {
    let available_files = files::Entity::find()
        .filter(files::Column::LibraryId.eq(library.id))
        .filter(files::Column::UnavailableAt.is_null())
        .order_by(files::Column::Id, Order::Asc)
        .all(pool)
        .await?;

    let mut parsed_files = Vec::with_capacity(available_files.len());
    for batch in available_files.chunks(PARSE_BATCH_SIZE) {
        let relative_paths = batch
            .iter()
            .map(|file| file.relative_path.clone())
            .collect::<Vec<_>>();
        let parsed_batch = parse_files(relative_paths).await;

        parsed_files.extend(
            batch
                .iter()
                .cloned()
                .zip(parsed_batch.into_iter())
                .collect::<Vec<(files::Model, ParsedFile)>>(),
        );
    }

    let derived = derive_library_media(library_root, &parsed_files);
    upsert_derived_media(pool, library.id, derived).await?;
    Ok(())
}

async fn upsert_derived_media(
    pool: &DatabaseConnection,
    library_id: i64,
    derived: derivation::DerivedLibraryMedia,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().timestamp();
    let txn = pool.begin().await?;

    if derived.roots.is_empty() {
        roots::Entity::delete_many()
            .filter(roots::Column::LibraryId.eq(library_id))
            .exec(&txn)
            .await?;
        txn.commit().await?;
        return Ok(());
    }

    for root in &derived.roots {
        roots::Entity::insert(roots::ActiveModel {
            id: Set(root.id.clone()),
            library_id: Set(library_id),
            kind: Set(root.kind),
            name: Set(root.name.clone()),
            match_candidates_json: Set(None),
            last_added_at: Set(root.last_added_at),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .on_conflict(
            OnConflict::column(roots::Column::Id)
                .update_columns([
                    roots::Column::LibraryId,
                    roots::Column::Kind,
                    roots::Column::Name,
                    roots::Column::LastAddedAt,
                    roots::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    }

    let root_ids = derived
        .roots
        .iter()
        .map(|root| root.id.clone())
        .collect::<Vec<_>>();

    roots::Entity::delete_many()
        .filter(roots::Column::LibraryId.eq(library_id))
        .filter(roots::Column::Id.is_not_in(root_ids.clone()))
        .exec(&txn)
        .await?;

    for season in &derived.seasons {
        seasons::Entity::insert(seasons::ActiveModel {
            id: Set(season.id.clone()),
            root_id: Set(season.root_id.clone()),
            season_number: Set(season.season_number),
            order: Set(season.order),
            name: Set(season.name.clone()),
            last_added_at: Set(season.last_added_at),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .on_conflict(
            OnConflict::column(seasons::Column::Id)
                .update_columns([
                    seasons::Column::RootId,
                    seasons::Column::SeasonNumber,
                    seasons::Column::Order,
                    seasons::Column::Name,
                    seasons::Column::LastAddedAt,
                    seasons::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    }

    let season_ids = derived
        .seasons
        .iter()
        .map(|season| season.id.clone())
        .collect::<Vec<_>>();

    if season_ids.is_empty() {
        seasons::Entity::delete_many()
            .filter(seasons::Column::RootId.is_in(root_ids.clone()))
            .exec(&txn)
            .await?;
    } else {
        seasons::Entity::delete_many()
            .filter(seasons::Column::RootId.is_in(root_ids.clone()))
            .filter(seasons::Column::Id.is_not_in(season_ids.clone()))
            .exec(&txn)
            .await?;
    }

    for item in &derived.items {
        items::Entity::insert(items::ActiveModel {
            id: Set(item.id.clone()),
            root_id: Set(item.root_id.clone()),
            season_id: Set(item.season_id.clone()),
            kind: Set(item.kind),
            episode_number: Set(item.episode_number),
            order: Set(item.order),
            name: Set(item.name.clone()),
            primary_file_id: Set(item.primary_file_id),
            last_added_at: Set(item.last_added_at),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .on_conflict(
            OnConflict::column(items::Column::Id)
                .update_columns([
                    items::Column::RootId,
                    items::Column::SeasonId,
                    items::Column::Kind,
                    items::Column::EpisodeNumber,
                    items::Column::Order,
                    items::Column::Name,
                    items::Column::PrimaryFileId,
                    items::Column::LastAddedAt,
                    items::Column::UpdatedAt,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;
    }

    let item_ids = derived
        .items
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();

    if item_ids.is_empty() {
        items::Entity::delete_many()
            .filter(items::Column::RootId.is_in(root_ids.clone()))
            .exec(&txn)
            .await?;
    } else {
        items::Entity::delete_many()
            .filter(items::Column::RootId.is_in(root_ids.clone()))
            .filter(items::Column::Id.is_not_in(item_ids.clone()))
            .exec(&txn)
            .await?;

        item_files::Entity::delete_many()
            .filter(item_files::Column::ItemId.is_in(item_ids.clone()))
            .exec(&txn)
            .await?;

        for item_file in &derived.item_files {
            item_files::Entity::insert(item_files::ActiveModel {
                item_id: Set(item_file.item_id.clone()),
                file_id: Set(item_file.file_id),
                order: Set(item_file.order),
                is_primary: Set(item_file.is_primary),
                created_at: Set(now),
                updated_at: Set(now),
            })
            .on_conflict(
                OnConflict::columns([item_files::Column::ItemId, item_files::Column::FileId])
                    .update_columns([
                        item_files::Column::Order,
                        item_files::Column::IsPrimary,
                        item_files::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await?;
        }
    }

    root_metadata::Entity::delete_many()
        .filter(root_metadata::Column::RootId.is_in(root_ids.clone()))
        .filter(root_metadata::Column::Source.eq(MetadataSource::Local))
        .exec(&txn)
        .await?;

    for root in &derived.roots {
        insert_local_root_metadata(
            &txn,
            &root.id,
            &root.name,
            root.imdb_id.clone(),
            root.tmdb_id,
            now,
        )
        .await?;
    }

    if !season_ids.is_empty() {
        season_metadata::Entity::delete_many()
            .filter(season_metadata::Column::SeasonId.is_in(season_ids.clone()))
            .filter(season_metadata::Column::Source.eq(MetadataSource::Local))
            .exec(&txn)
            .await?;

        for season in &derived.seasons {
            insert_local_season_metadata(&txn, &season.root_id, &season.id, &season.name, now)
                .await?;
        }
    }

    if !item_ids.is_empty() {
        item_metadata::Entity::delete_many()
            .filter(item_metadata::Column::ItemId.is_in(item_ids.clone()))
            .filter(item_metadata::Column::Source.eq(MetadataSource::Local))
            .exec(&txn)
            .await?;

        for item in &derived.items {
            insert_local_item_metadata(&txn, &item.root_id, &item.id, &item.name, now).await?;
        }
    }

    txn.commit().await?;
    Ok(())
}
