use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::Notify;

const JOB_LOCK_COOLDOWN_MS: u64 = 15 * 60 * 1_000;

#[derive(Clone, Default)]
pub struct JobLock {
    active_blocks: Arc<AtomicUsize>,
    blocked_until_ms: Arc<AtomicU64>,
    changed: Arc<Notify>,
}

impl JobLock {
    pub fn take_block(&self) -> JobLockGuard {
        self.active_blocks.fetch_add(1, Ordering::SeqCst);
        self.changed.notify_waiters();
        JobLockGuard {
            job_lock: self.clone(),
        }
    }

    pub fn is_blocked(&self) -> bool {
        self.active_blocks.load(Ordering::SeqCst) > 0
            || current_time_ms() < self.blocked_until_ms.load(Ordering::SeqCst)
    }

    pub async fn wait_until_unblocked(&self) {
        loop {
            let notified = self.changed.notified();
            let active_blocks = self.active_blocks.load(Ordering::SeqCst);
            let blocked_until_ms = self.blocked_until_ms.load(Ordering::SeqCst);
            let now_ms = current_time_ms();
            if active_blocks == 0 && now_ms >= blocked_until_ms {
                return;
            }

            if active_blocks == 0 && blocked_until_ms > now_ms {
                let wait_ms = blocked_until_ms.saturating_sub(now_ms);
                tokio::select! {
                    _ = notified => {},
                    _ = tokio::time::sleep(Duration::from_millis(wait_ms)) => {},
                }
            } else {
                notified.await;
            }
        }
    }

    pub async fn wait_until_blocked(&self) {
        loop {
            if self.is_blocked() {
                return;
            }

            self.changed.notified().await;
        }
    }

    fn release_block(&self) {
        self.active_blocks.fetch_sub(1, Ordering::SeqCst);
        self.blocked_until_ms
            .store(current_time_ms() + JOB_LOCK_COOLDOWN_MS, Ordering::SeqCst);
        self.changed.notify_waiters();
    }
}

pub struct JobLockGuard {
    job_lock: JobLock,
}

impl Drop for JobLockGuard {
    fn drop(&mut self) {
        self.job_lock.release_block();
    }
}

fn current_time_ms() -> u64 {
    chrono::Utc::now().timestamp_millis().max(0) as u64
}
