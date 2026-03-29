use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicU64},
    time::Duration,
};
use tokio::sync::{Mutex, Notify, OwnedSemaphorePermit};
use tokio_util::sync::CancellationToken;

pub struct JobSemaphore {
    semaphore: Arc<tokio::sync::Semaphore>,
    active_blocks: Arc<AtomicU64>,
    current_id: AtomicU64,
    cancellation_tokens: Arc<Mutex<HashMap<u64, CancellationToken>>>,
    no_blocks_notify: Arc<Notify>,
}

impl JobSemaphore {
    pub fn new() -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(1)),
            active_blocks: Arc::new(AtomicU64::new(0)),
            current_id: AtomicU64::new(0),
            cancellation_tokens: Arc::new(Mutex::new(HashMap::new())),
            no_blocks_notify: Arc::new(Notify::new()),
        }
    }

    pub async fn push_lock(&self, grace_period: Duration) -> HeavyJobPreventionGuard {
        // we have to do this up front so no new jobs can acquire a lease while we are acquiring the
        // guard + cancelling existing jobs.
        let tokens = self.cancellation_tokens.lock().await;
        let prev_active = self
            .active_blocks
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if prev_active == 0 {
            tracing::info!(?grace_period, "blocking heavy jobs from running")
        }

        for token in tokens.values() {
            token.cancel();
        }

        HeavyJobPreventionGuard {
            active_blocks: self.active_blocks.clone(),
            grace_period,
            no_blocks_notify: self.no_blocks_notify.clone(),
        }
    }

    pub async fn acquire_lease(&self, is_heavy: bool) -> JobLease {
        // normal jobs are not cancellable and do not have any restrictions,
        // no need to do anything.
        if !is_heavy {
            return JobLease::new_blank();
        }

        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        let cancel_token = CancellationToken::new();
        let job_id = self
            .current_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        loop {
            let mut tokens = self.cancellation_tokens.lock().await;
            if self.active_blocks.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                drop(tokens);
                tokio::select! {
                    _ = self.no_blocks_notify.notified() => {},
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                }

                continue;
            }

            tokens.insert(job_id, cancel_token.clone());
            drop(tokens);

            return JobLease::new_cancellable(
                job_id,
                permit,
                cancel_token,
                self.cancellation_tokens.clone(),
            );
        }
    }
}

/// JobBlock guards against heavy jobs running while held + a grace period
/// Heavy jobs may still run while held (on demand jobs, jobs that cannot be cancelled)
pub struct HeavyJobPreventionGuard {
    active_blocks: Arc<AtomicU64>,
    grace_period: Duration,
    no_blocks_notify: Arc<Notify>,
}

impl Drop for HeavyJobPreventionGuard {
    fn drop(&mut self) {
        let active_blocks = self.active_blocks.clone();
        let grace_period = self.grace_period;
        let no_blocks_notify = self.no_blocks_notify.clone();
        tokio::spawn(async move {
            // wait for the grace period before releasing the block, to stop jobs running immediately after a block is lifted
            tokio::time::sleep(grace_period).await;
            let prev = active_blocks.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            if prev == 1 {
                // notify any waiting tasks that the block has been lifted
                no_blocks_notify.notify_waiters();
                tracing::info!("heavy job block lifted");
            }
        });
    }
}

pub struct JobLease {
    _permit: Option<OwnedSemaphorePermit>,
    id: Option<u64>,
    cancel: Option<CancellationToken>,
    cancellation_tokens: Option<Arc<Mutex<HashMap<u64, CancellationToken>>>>,
}

impl JobLease {
    pub(super) fn new_blank() -> Self {
        Self {
            _permit: None,
            cancel: None,
            cancellation_tokens: None,
            id: None,
        }
    }

    pub(super) fn new_cancellable(
        id: u64,
        permit: OwnedSemaphorePermit,
        cancel: CancellationToken,
        cancellation_tokens: Arc<Mutex<HashMap<u64, CancellationToken>>>,
    ) -> Self {
        Self {
            id: Some(id),
            _permit: Some(permit),
            cancel: Some(cancel),
            cancellation_tokens: Some(cancellation_tokens),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.as_ref().map_or(false, |c| c.is_cancelled())
    }

    pub fn get_cancellation_token(&self) -> Option<&CancellationToken> {
        self.cancel.as_ref()
    }
}

impl Drop for JobLease {
    fn drop(&mut self) {
        let id = self.id.take();
        let cancellation_tokens = self.cancellation_tokens.take();
        if let (Some(id), Some(cancellation_tokens)) = (id, cancellation_tokens) {
            tokio::spawn(async move {
                let mut tokens = cancellation_tokens.lock().await;
                tokens.remove(&id);
            });
        }
    }
}
