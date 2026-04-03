use crate::content_update::CONTENT_UPDATE;
use crate::entities::{files, libraries, nodes};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use std::collections::HashSet;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;

const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

pub async fn start_cleanup_worker(
    pool: DatabaseConnection,
    startup_scans_complete: CancellationToken,
) -> anyhow::Result<()> {
    // Let startup scans establish fresh availability state before we prune anything.
    startup_scans_complete.cancelled().await;

    loop {
        prune_stale_unavailable(&pool).await?;
        sleep(CLEANUP_INTERVAL).await;
    }
}

async fn prune_stale_unavailable(pool: &DatabaseConnection) -> anyhow::Result<()> {
    let Some(cutoff) = chrono::Utc::now()
        .checked_sub_months(chrono::Months::new(3))
        .map(|time| time.timestamp())
    else {
        return Ok(());
    };

    let available_library_ids: Vec<String> = libraries::Entity::find()
        .filter(libraries::Column::UnavailableAt.is_null())
        .select_only()
        .column(libraries::Column::Id)
        .into_tuple::<String>()
        .all(pool)
        .await?;
    if available_library_ids.is_empty() {
        return Ok(());
    }

    let stale_nodes = nodes::Entity::find()
        .filter(nodes::Column::LibraryId.is_in(available_library_ids.clone()))
        .filter(nodes::Column::UnavailableAt.is_not_null())
        .filter(nodes::Column::UnavailableAt.lte(cutoff))
        .all(pool)
        .await?;
    let stale_node_ids = stale_nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();
    let stale_subtree_root_ids = stale_nodes
        .iter()
        .filter(|node| {
            node.parent_id
                .as_ref()
                .is_none_or(|parent_id| !stale_node_ids.contains(parent_id))
        })
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();

    let mut deleted_anything = false;
    if !stale_subtree_root_ids.is_empty() {
        nodes::Entity::delete_many()
            .filter(nodes::Column::Id.is_in(stale_subtree_root_ids))
            .exec(pool)
            .await?;
        deleted_anything = true;
    }

    let stale_file_ids: Vec<String> = files::Entity::find()
        .filter(files::Column::LibraryId.is_in(available_library_ids))
        .filter(files::Column::UnavailableAt.is_not_null())
        .filter(files::Column::UnavailableAt.lte(cutoff))
        .select_only()
        .column(files::Column::Id)
        .into_tuple::<String>()
        .all(pool)
        .await?;

    if !stale_file_ids.is_empty() {
        files::Entity::delete_many()
            .filter(files::Column::Id.is_in(stale_file_ids))
            .exec(pool)
            .await?;
        deleted_anything = true;
    }

    if deleted_anything {
        CONTENT_UPDATE.emit();
    }

    Ok(())
}
