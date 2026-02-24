use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub type SharedSyncVersion = Arc<AtomicU64>;

pub fn new_sync_version() -> SharedSyncVersion {
    Arc::new(AtomicU64::new(0))
}

pub fn read_sync_version(sync_version: &AtomicU64) -> u64 {
    sync_version.load(Ordering::Relaxed)
}

pub fn bump_sync_version(sync_version: &AtomicU64) -> u64 {
    sync_version.fetch_add(1, Ordering::Relaxed) + 1
}
