use crate::entities::item_node_matches;
use std::collections::HashMap;

pub const MAX_ROOTS_PER_TICK: usize = 25;
pub const MAX_ITEMS_PER_TICK: usize = 120;
pub const MAX_HINT_FILES: u64 = 150;
pub const RETRY_BACKOFF_SECONDS: &[i64] = &[5 * 60, 30 * 60, 6 * 60 * 60, 24 * 60 * 60];

pub struct RootMatchRowInput {
    pub root_id: String,
    pub provider_id: String,
    pub status: crate::entities::node_match_status::NodeMatchStatus,
    pub last_attempted_at: Option<i64>,
    pub last_added_at: Option<i64>,
    pub last_error_message: Option<String>,
    pub retry_after: Option<i64>,
    pub attempts: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct ItemMatchRowInput {
    pub root_id: String,
    pub item_id: String,
    pub provider_id: String,
    pub status: crate::entities::node_match_status::NodeMatchStatus,
    pub last_attempted_at: Option<i64>,
    pub last_added_at: Option<i64>,
    pub last_error_message: Option<String>,
    pub retry_after: Option<i64>,
    pub attempts: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn retry_backoff_seconds(attempts: i64) -> i64 {
    let index = attempts.saturating_sub(1) as usize;
    RETRY_BACKOFF_SECONDS
        .get(index)
        .copied()
        .unwrap_or(*RETRY_BACKOFF_SECONDS.last().unwrap_or(&86_400))
}

pub fn next_attempts(
    existing_rows: &HashMap<String, item_node_matches::Model>,
    item_id: &str,
) -> i64 {
    existing_rows
        .get(item_id)
        .map(|row| row.attempts + 1)
        .unwrap_or(1)
}
