use lazy_static::lazy_static;
use std::cmp::{max, min};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Notify, broadcast};
use tokio::time::{Duration, Instant};

const CHANNEL_CAPACITY: usize = 32;
const DEBOUNCE_WINDOW: Duration = Duration::from_millis(500);
const MIN_EMIT_INTERVAL: Duration = Duration::from_secs(5);
const MAX_DEBOUNCE_WINDOW: Duration = Duration::from_secs(60);

lazy_static! {
    pub static ref CONTENT_UPDATE: ContentUpdateEmitter = ContentUpdateEmitter::new();
}

pub struct ContentUpdateEmitter {
    state: Mutex<ContentUpdateState>,
    notify: Notify,
    sender: broadcast::Sender<()>,
    started: AtomicBool,
}

#[derive(Default)]
struct ContentUpdateState {
    pending_since: Option<Instant>,
    latest_request_at: Option<Instant>,
    last_emitted_at: Option<Instant>,
}

impl ContentUpdateState {
    fn next_deadline(&self) -> Option<Instant> {
        let pending_since = self.pending_since?;
        let latest_request_at = self.latest_request_at?;
        let min_emit_at = self
            .last_emitted_at
            .map(|last_emitted_at| last_emitted_at + MIN_EMIT_INTERVAL)
            .unwrap_or(pending_since);
        let debounce_deadline = latest_request_at + DEBOUNCE_WINDOW;
        let max_debounce_deadline = pending_since + MAX_DEBOUNCE_WINDOW;

        Some(max(
            min_emit_at,
            min(debounce_deadline, max_debounce_deadline),
        ))
    }

    fn mark_pending(&mut self, now: Instant) {
        if self.pending_since.is_none() {
            self.pending_since = Some(now);
        }
        self.latest_request_at = Some(now);
    }

    fn mark_emitted(&mut self, now: Instant) {
        self.pending_since = None;
        self.latest_request_at = None;
        self.last_emitted_at = Some(now);
    }
}

impl ContentUpdateEmitter {
    fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);

        Self {
            state: Mutex::new(ContentUpdateState::default()),
            notify: Notify::new(),
            sender,
            started: AtomicBool::new(false),
        }
    }

    pub fn start(&'static self) {
        if self.started.swap(true, Ordering::AcqRel) {
            return;
        }

        tokio::spawn(async move {
            self.run().await;
        });
    }

    pub fn emit(&self) {
        let now = Instant::now();
        self.state.lock().unwrap().mark_pending(now);
        self.notify.notify_one();
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    async fn run(&self) {
        loop {
            let next_deadline = self.state.lock().unwrap().next_deadline();

            match next_deadline {
                Some(deadline) => {
                    tokio::select! {
                        _ = tokio::time::sleep_until(deadline) => {
                            let should_emit = {
                                let now = Instant::now();
                                let mut state = self.state.lock().unwrap();
                                let ready = state
                                    .next_deadline()
                                    .is_some_and(|current_deadline| current_deadline <= now);

                                if ready {
                                    state.mark_emitted(now);
                                }

                                ready
                            };

                            if should_emit {
                                let _ = self.sender.send(());
                            }
                        }
                        _ = self.notify.notified() => {}
                    }
                }
                None => self.notify.notified().await,
            }
        }
    }
}
