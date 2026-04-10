use crate::{session::Session, types::SessionOptions};
use lyra_ids::generate_prefixed_ulid;
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, watch},
    task::JoinHandle,
    time::MissedTickBehavior,
};

const DEFAULT_SWEEP_INTERVAL: Duration = Duration::from_secs(60);

struct SessionManagerInner {
    root_work_dir: PathBuf,
    idle_timeout: Duration,
    sessions: Mutex<HashMap<String, Arc<Session>>>,
    session_count_tx: watch::Sender<usize>,
}

pub struct SessionManager {
    inner: Arc<SessionManagerInner>,
    sweeper_handle: JoinHandle<()>,
}

impl SessionManager {
    pub async fn new(root_work_dir: PathBuf, idle_timeout: Duration) -> anyhow::Result<Self> {
        tokio::fs::create_dir_all(&root_work_dir).await?;

        let inner = Arc::new(SessionManagerInner {
            root_work_dir,
            idle_timeout,
            sessions: Mutex::new(HashMap::new()),
            session_count_tx: watch::channel(0).0,
        });
        let sweeper_handle = spawn_sweeper(inner.clone());

        Ok(Self {
            inner,
            sweeper_handle,
        })
    }

    pub async fn create(&self, options: SessionOptions) -> anyhow::Result<Arc<Session>> {
        loop {
            let session_id = generate_prefixed_ulid("ps");
            if self.session(&session_id).await.is_none() {
                return self
                    .insert_session(session_id, options.clone(), false)
                    .await;
            }
        }
    }

    pub async fn get_or_create(
        &self,
        session_id: &str,
        options: SessionOptions,
    ) -> anyhow::Result<Arc<Session>> {
        if let Some(existing) = self.session(session_id).await {
            anyhow::ensure!(
                existing.spec() == &options.spec,
                "session {} already exists with different options",
                session_id
            );
            existing.touch();
            return Ok(existing);
        }

        self.insert_session(session_id.to_string(), options, true)
            .await
    }

    pub async fn session(&self, session_id: &str) -> Option<Arc<Session>> {
        self.inner.sessions.lock().await.get(session_id).cloned()
    }

    pub fn subscribe_session_count(&self) -> watch::Receiver<usize> {
        self.inner.session_count_tx.subscribe()
    }

    pub async fn session_count(&self) -> usize {
        self.inner.sessions.lock().await.len()
    }

    pub async fn attach_player(
        &self,
        session_id: &str,
        player_id: String,
    ) -> anyhow::Result<Arc<Session>> {
        let session = self
            .session(session_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("unknown session {}", session_id))?;
        session.add_player(player_id).await;
        Ok(session)
    }

    pub async fn detach_player(&self, session_id: &str, player_id: &str) -> anyhow::Result<()> {
        let session = self.session(session_id).await;
        let Some(session) = session else {
            return Ok(());
        };

        if session.remove_player(player_id).await {
            self.remove_session(session_id).await?;
        }

        Ok(())
    }

    pub async fn prune_idle_sessions(&self) -> anyhow::Result<()> {
        let idle_timeout = self.inner.idle_timeout;
        let candidates = self
            .inner
            .sessions
            .lock()
            .await
            .iter()
            .filter_map(|(session_id, session)| {
                session
                    .is_idle_for(idle_timeout)
                    .then_some(session_id.clone())
            })
            .collect::<Vec<_>>();

        for session_id in candidates {
            self.remove_session(&session_id).await?;
        }

        Ok(())
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.sweeper_handle.abort();
        let sessions = self
            .inner
            .sessions
            .lock()
            .await
            .drain()
            .map(|(_, session)| session)
            .collect::<Vec<_>>();
        let _ = self.inner.session_count_tx.send(0);

        for session in sessions {
            session.shutdown().await?;
        }

        Ok(())
    }

    async fn insert_session(
        &self,
        session_id: String,
        options: SessionOptions,
        overwrite_existing: bool,
    ) -> anyhow::Result<Arc<Session>> {
        let work_dir = self.inner.root_work_dir.join(&session_id);
        tokio::fs::create_dir_all(&work_dir).await?;

        let session = Arc::new(Session::new(session_id.clone(), work_dir, options)?);
        let mut sessions = self.inner.sessions.lock().await;
        if overwrite_existing {
            if let Some(existing) = sessions.get(&session_id) {
                anyhow::ensure!(
                    existing.spec() == session.spec(),
                    "session {} already exists with different options",
                    session_id
                );
                return Ok(existing.clone());
            }
        }
        sessions.insert(session_id, session.clone());
        let _ = self.inner.session_count_tx.send(sessions.len());
        Ok(session)
    }

    async fn remove_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.inner.sessions.lock().await;
        let session = sessions.remove(session_id);
        let _ = self.inner.session_count_tx.send(sessions.len());
        drop(sessions);
        if let Some(session) = session {
            session.shutdown().await?;
        }
        Ok(())
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.sweeper_handle.abort();
    }
}

fn spawn_sweeper(inner: Arc<SessionManagerInner>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(DEFAULT_SWEEP_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            let candidates = inner
                .sessions
                .lock()
                .await
                .iter()
                .filter_map(|(session_id, session)| {
                    session
                        .is_idle_for(inner.idle_timeout)
                        .then_some(session_id.clone())
                })
                .collect::<Vec<_>>();

            for session_id in candidates {
                let mut sessions = inner.sessions.lock().await;
                let session = sessions.remove(&session_id);
                let _ = inner.session_count_tx.send(sessions.len());
                drop(sessions);
                if let Some(session) = session {
                    let _ = session.shutdown().await;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::SessionManager;
    use crate::types::{SessionOptions, SessionSpec, VideoProfileSelection};
    use lyra_probe::{Codec, ProbeData, Stream, StreamDetails, StreamDisposition, VideoKeyframes};
    use std::{sync::Arc, time::Duration};

    fn test_probe() -> ProbeData {
        ProbeData {
            duration_secs: Some(60.0),
            overall_bit_rate: None,
            streams: vec![Stream {
                index: 0,
                codec: Codec::VideoH264,
                display_name: None,
                original_title: None,
                bit_rate: None,
                language_bcp47: None,
                disposition: StreamDisposition::DEFAULT,
                details: StreamDetails::Video {
                    width: 1920,
                    height: 1080,
                    time_base_num: 1,
                    time_base_den: 1_000,
                    frame_rate: Some(24.0),
                    profile: None,
                    level: None,
                    codec_tag_string: None,
                    bit_depth: None,
                    hdr_format: None,
                },
            }],
        }
    }

    fn test_options() -> SessionOptions {
        SessionOptions {
            spec: SessionSpec {
                file_path: "/tmp/input.mkv".into(),
                video: VideoProfileSelection {
                    stream_index: 0,
                    profile_id: "copy".to_string(),
                },
                audio: None,
            },
            probe: test_probe(),
            keyframes: Some(VideoKeyframes::new(0, 1, 1_000, vec![0, 6_000, 12_000]).unwrap()),
        }
    }

    #[tokio::test]
    async fn get_or_create_reuses_matching_session() {
        let root = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(root.path().to_path_buf(), Duration::from_secs(900))
            .await
            .unwrap();

        let first = manager
            .get_or_create("ps_test", test_options())
            .await
            .unwrap();
        let second = manager
            .get_or_create("ps_test", test_options())
            .await
            .unwrap();

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[tokio::test]
    async fn detaching_last_player_removes_the_session() {
        let root = tempfile::tempdir().unwrap();
        let manager = SessionManager::new(root.path().to_path_buf(), Duration::from_secs(900))
            .await
            .unwrap();

        manager
            .get_or_create("ps_test", test_options())
            .await
            .unwrap();
        manager
            .attach_player("ps_test", "player-1".to_string())
            .await
            .unwrap();
        manager.detach_player("ps_test", "player-1").await.unwrap();

        assert!(manager.session("ps_test").await.is_none());
    }
}
