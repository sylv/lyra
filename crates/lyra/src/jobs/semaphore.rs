use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;

#[derive(Default)]
struct HeavyJobState {
    active_blocks: u64,
    next_job_id: u64,
    active_job_id: Option<u64>,
    current_cancel: Option<CancellationToken>,
}

pub struct HeavyJobController {
    state: Arc<Mutex<HeavyJobState>>,
    no_blocks_notify: Arc<Notify>,
}

impl HeavyJobController {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HeavyJobState::default())),
            no_blocks_notify: Arc::new(Notify::new()),
        }
    }

    pub async fn push_block(&self, grace_period: Duration) -> HeavyJobBlockGuard {
        let mut state = self.state.lock().await;
        let prev_active = state.active_blocks;
        state.active_blocks += 1;

        if prev_active == 0 {
            tracing::info!(?grace_period, "blocking heavy jobs from running");
        }

        if let Some(token) = &state.current_cancel {
            token.cancel();
        }

        drop(state);

        HeavyJobBlockGuard {
            state: self.state.clone(),
            grace_period,
            no_blocks_notify: self.no_blocks_notify.clone(),
        }
    }

    pub async fn acquire_background_lease(&self) -> JobLease {
        loop {
            let mut state = self.state.lock().await;
            if state.active_blocks > 0 {
                drop(state);
                tokio::select! {
                    _ = self.no_blocks_notify.notified() => {},
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                }

                continue;
            }

            let job_id = state.next_job_id;
            state.next_job_id += 1;
            let cancel_token = CancellationToken::new();
            state.active_job_id = Some(job_id);
            state.current_cancel = Some(cancel_token.clone());
            drop(state);

            return JobLease::new_cancellable(self.state.clone(), job_id, cancel_token);
        }
    }
}

/// HeavyJobBlockGuard prevents background heavy jobs from starting while held and for a short
/// grace period afterwards so playback-triggered work does not immediately contend again.
pub struct HeavyJobBlockGuard {
    state: Arc<Mutex<HeavyJobState>>,
    grace_period: Duration,
    no_blocks_notify: Arc<Notify>,
}

impl Drop for HeavyJobBlockGuard {
    fn drop(&mut self) {
        let state = self.state.clone();
        let grace_period = self.grace_period;
        let no_blocks_notify = self.no_blocks_notify.clone();
        tokio::spawn(async move {
            tokio::time::sleep(grace_period).await;

            let mut state = state.lock().await;
            state.active_blocks = state.active_blocks.saturating_sub(1);
            let should_notify = state.active_blocks == 0;
            drop(state);

            if should_notify {
                no_blocks_notify.notify_waiters();
                tracing::info!("heavy job block lifted");
            }
        });
    }
}

pub struct JobLease {
    job_id: Option<u64>,
    cancel: Option<CancellationToken>,
    state: Option<Arc<Mutex<HeavyJobState>>>,
}

impl JobLease {
    pub(super) fn new_blank() -> Self {
        Self {
            job_id: None,
            cancel: None,
            state: None,
        }
    }

    fn new_cancellable(
        state: Arc<Mutex<HeavyJobState>>,
        job_id: u64,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            job_id: Some(job_id),
            cancel: Some(cancel),
            state: Some(state),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.as_ref().is_some_and(|c| c.is_cancelled())
    }

    pub fn get_cancellation_token(&self) -> Option<&CancellationToken> {
        self.cancel.as_ref()
    }
}

impl Drop for JobLease {
    fn drop(&mut self) {
        let job_id = self.job_id.take();
        let _cancel = self.cancel.take();
        let state = self.state.take();
        if let (Some(job_id), Some(state)) = (job_id, state) {
            tokio::spawn(async move {
                let mut state = state.lock().await;
                if state.active_job_id == Some(job_id) {
                    state.active_job_id = None;
                    state.current_cancel = None;
                }
            });
        }
    }
}
