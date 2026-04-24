use crate::{
    auth::{RequestAuth, ensure_library_access},
    entities::{
        files, nodes,
        users::{self, UserPerms},
    },
    graphql::properties::NodeProperties,
};
use async_graphql::{ComplexObject, Context, Enum, InputObject, SimpleObject};
use chrono::Utc;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, broadcast};

const HEARTBEAT_INTERVAL_MS: i64 = 3_000;
const HEARTBEAT_TIMEOUT_MS: i64 = 12_000;
const INACTIVE_ONLY_DELETE_AFTER_MS: i64 = 5 * 60 * 1_000;
const SESSION_CHANNEL_CAPACITY: usize = 32;

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum WatchSessionMode {
    Advisory,
    Synced,
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum EffectiveWatchSessionState {
    Playing,
    Paused,
    Buffering,
    InactivePlayers,
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum WatchSessionActionKind {
    Play,
    Pause,
    Seek,
    SwitchItem,
    RemovePlayer,
}

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum WatchSessionIntent {
    Playing,
    Paused,
}

#[derive(Clone, Debug, InputObject)]
pub struct WatchSessionRecoveryInput {
    pub node_id: String,
    pub file_id: String,
    pub intent: WatchSessionIntent,
    pub base_position_ms: i32,
    pub base_time_ms: f64,
}

#[derive(Clone, Debug, InputObject)]
pub struct WatchSessionHeartbeatInput {
    pub session_id: String,
    pub player_id: String,
    pub is_buffering: bool,
    pub base_position_ms: i32,
    pub base_time_ms: f64,
    pub recovery: WatchSessionRecoveryInput,
}

#[derive(Clone, Debug, InputObject)]
pub struct WatchSessionActionInput {
    pub session_id: String,
    pub player_id: String,
    pub kind: WatchSessionActionKind,
    pub position_ms: Option<i32>,
    pub node_id: Option<String>,
    pub target_player_id: Option<String>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct WatchSessionPlayer {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub display_username: String,
    #[graphql(skip)]
    pub is_buffering: bool,
    pub base_position_ms: i32,
    #[graphql(skip)]
    pub base_time_ms: i64,
    #[graphql(skip)]
    pub last_report_ms: i64,
    pub joined_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(name = "WatchSession", complex)]
pub struct WatchSession {
    pub id: String,
    pub node_id: String,
    pub file_id: String,
    pub intent: WatchSessionIntent,
    pub base_position_ms: i32,
    #[graphql(skip)]
    pub base_time_ms: i64,
    pub revision: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub players: Vec<WatchSessionPlayer>,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct WatchSessionBeacon {
    pub session_id: String,
    pub node_id: String,
    pub file_id: String,
    pub mode: WatchSessionMode,
    pub intent: WatchSessionIntent,
    pub effective_state: EffectiveWatchSessionState,
    pub base_position_ms: i32,
    #[graphql(skip)]
    pub base_time_ms: i64,
    pub revision: i32,
    pub players: Vec<WatchSessionPlayer>,
}

#[ComplexObject]
impl WatchSessionBeacon {
    async fn base_time_ms(&self) -> f64 {
        self.base_time_ms as f64
    }
}

#[derive(Clone)]
pub struct WatchSessionRegistry {
    pool: DatabaseConnection,
    runtimes: Arc<Mutex<HashMap<String, Arc<WatchSessionRuntime>>>>,
}

struct WatchSessionRuntime {
    session: Mutex<WatchSession>,
    sender: broadcast::Sender<WatchSessionBeacon>,
}

impl WatchSessionRuntime {
    fn new(session: WatchSession) -> Self {
        let (sender, _) = broadcast::channel(SESSION_CHANNEL_CAPACITY);
        Self {
            session: Mutex::new(session),
            sender,
        }
    }
}

impl WatchSessionRegistry {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self {
            pool,
            runtimes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(&self) {
        let registry = self.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_millis(HEARTBEAT_INTERVAL_MS as u64));

            loop {
                interval.tick().await;
                if let Err(error) = registry.reconcile_all().await {
                    tracing::warn!(error = ?error, "watch session reconcile tick failed");
                }
            }
        });
    }

    pub async fn session_for_view(
        &self,
        auth: &RequestAuth,
        session_id: &str,
    ) -> Result<Option<WatchSession>, async_graphql::Error> {
        let Some(runtime) = self.get_runtime(session_id).await else {
            return Ok(None);
        };

        let session = runtime.session.lock().await.clone();
        self.ensure_session_access(auth, &session).await?;
        Ok(Some(session))
    }

    pub async fn sessions_snapshot(&self) -> Vec<WatchSession> {
        let runtimes = self
            .runtimes
            .lock()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();

        let mut sessions = Vec::with_capacity(runtimes.len());
        for runtime in runtimes {
            sessions.push(runtime.session.lock().await.clone());
        }

        sessions.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| b.created_at.cmp(&a.created_at))
                .then_with(|| a.id.cmp(&b.id))
        });
        sessions
    }

    pub async fn subscribe_for_player(
        &self,
        auth: &RequestAuth,
        session_id: &str,
        player_id: &str,
    ) -> Result<broadcast::Receiver<WatchSessionBeacon>, async_graphql::Error> {
        let user = auth.get_user_or_err()?;
        let runtime = self
            .get_runtime(session_id)
            .await
            .ok_or_else(|| async_graphql::Error::new("Watch session not found"))?;
        let session = runtime.session.lock().await.clone();
        self.ensure_session_access(auth, &session).await?;

        let player = session
            .players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| async_graphql::Error::new("Watch session player not found"))?;
        if player.user_id != user.id {
            return Err(async_graphql::Error::new("Watch session player not found"));
        }

        Ok(runtime.sender.subscribe())
    }

    pub async fn leave_session(
        &self,
        auth: &RequestAuth,
        session_id: &str,
        player_id: &str,
    ) -> Result<bool, async_graphql::Error> {
        let user = auth.get_user_or_err()?;
        let Some(runtime) = self.get_runtime(session_id).await else {
            return Ok(true);
        };

        let now_s = Utc::now().timestamp();
        let beacon = {
            let mut session = runtime.session.lock().await;
            self.ensure_session_access(auth, &session).await?;

            let Some(player_index) = session
                .players
                .iter()
                .position(|player| player.id == player_id)
            else {
                return Ok(true);
            };
            if session.players[player_index].user_id != user.id {
                return Err(async_graphql::Error::new(
                    "You can only leave your own watch session player",
                ));
            }

            session.players.remove(player_index);
            if session.players.is_empty() {
                None
            } else {
                session.revision = session.revision.saturating_add(1);
                session.updated_at = now_s;
                Some(build_beacon(session.clone(), current_time_ms()))
            }
        };

        match beacon {
            Some(beacon) => {
                let _ = runtime.sender.send(beacon);
            }
            None => {
                self.remove_runtime(session_id).await;
            }
        }

        Ok(true)
    }

    pub async fn heartbeat(
        &self,
        auth: &RequestAuth,
        input: WatchSessionHeartbeatInput,
    ) -> Result<WatchSessionBeacon, async_graphql::Error> {
        let user = auth.get_user_or_err()?;
        let now_s = Utc::now().timestamp();
        let now_ms = current_time_ms();
        let (runtime, created_session) = self.get_or_create_runtime(auth, &input, now_s).await?;

        let (beacon, should_broadcast) = {
            let mut session = runtime.session.lock().await;
            self.ensure_session_access(auth, &session).await?;

            let mut should_broadcast = created_session;
            let player_index = session
                .players
                .iter()
                .position(|player| player.id == input.player_id);
            match player_index {
                Some(player_index) => {
                    let player = &mut session.players[player_index];
                    if player.user_id != user.id {
                        return Err(async_graphql::Error::new("Watch session player not found"));
                    }

                    player.display_username = user.username.clone();
                    let was_inactive = is_player_inactive(player, now_ms);
                    let buffering_changed = player.is_buffering != input.is_buffering;
                    player.is_buffering = input.is_buffering;
                    player.base_position_ms = input.base_position_ms.max(0);
                    player.base_time_ms = input.base_time_ms as i64;
                    player.last_report_ms = now_ms;
                    player.updated_at = now_s;

                    if was_inactive || buffering_changed {
                        session.revision = session.revision.saturating_add(1);
                        should_broadcast = true;
                    }
                }
                None => {
                    let had_players = !session.players.is_empty();
                    session.players.push(WatchSessionPlayer {
                        id: input.player_id.clone(),
                        session_id: input.session_id.clone(),
                        user_id: user.id.clone(),
                        display_username: user.username.clone(),
                        is_buffering: input.is_buffering,
                        base_position_ms: input.base_position_ms.max(0),
                        base_time_ms: input.base_time_ms as i64,
                        last_report_ms: now_ms,
                        joined_at: now_s,
                        updated_at: now_s,
                    });

                    if had_players {
                        session.revision = session.revision.saturating_add(1);
                    }
                    should_broadcast = true;
                }
            }

            session.updated_at = now_s;
            (build_beacon(session.clone(), now_ms), should_broadcast)
        };

        if should_broadcast {
            let _ = runtime.sender.send(beacon.clone());
        }

        Ok(beacon)
    }

    pub async fn apply_action(
        &self,
        auth: &RequestAuth,
        input: WatchSessionActionInput,
    ) -> Result<WatchSessionBeacon, async_graphql::Error> {
        let user = auth.get_user_or_err()?;
        let runtime = self
            .get_runtime(&input.session_id)
            .await
            .ok_or_else(|| async_graphql::Error::new("Watch session not found"))?;

        let now_s = Utc::now().timestamp();
        let now_ms = current_time_ms();
        let beacon = {
            let mut session = runtime.session.lock().await;
            self.ensure_session_access(auth, &session).await?;

            let player_index = session
                .players
                .iter()
                .position(|player| player.id == input.player_id)
                .ok_or_else(|| async_graphql::Error::new("Watch session player not found"))?;
            let player = session.players[player_index].clone();
            if player.user_id != user.id {
                return Err(async_graphql::Error::new("Watch session player not found"));
            }

            match input.kind {
                WatchSessionActionKind::Play => {
                    let position_ms = input.position_ms.ok_or_else(|| {
                        async_graphql::Error::new("positionMs is required for PLAY")
                    })?;
                    update_session_transport(
                        &mut session,
                        Some(WatchSessionIntent::Playing),
                        position_ms,
                        now_ms,
                        now_s,
                    );
                }
                WatchSessionActionKind::Pause => {
                    let position_ms = input.position_ms.ok_or_else(|| {
                        async_graphql::Error::new("positionMs is required for PAUSE")
                    })?;
                    update_session_transport(
                        &mut session,
                        Some(WatchSessionIntent::Paused),
                        position_ms,
                        now_ms,
                        now_s,
                    );
                }
                WatchSessionActionKind::Seek => {
                    let position_ms = input.position_ms.ok_or_else(|| {
                        async_graphql::Error::new("positionMs is required for SEEK")
                    })?;
                    update_session_transport(&mut session, None, position_ms, now_ms, now_s);
                }
                WatchSessionActionKind::SwitchItem => {
                    let node_id = input.node_id.as_deref().ok_or_else(|| {
                        async_graphql::Error::new("nodeId is required for SWITCH_ITEM")
                    })?;
                    let node = self.resolve_target_node(auth, node_id).await?;
                    let file = self.resolve_target_file(&node).await?;
                    session.node_id = node.id;
                    session.file_id = file.id;
                    session.intent = WatchSessionIntent::Paused;
                    session.base_position_ms = 0;
                    session.base_time_ms = now_ms;
                    session.revision = session.revision.saturating_add(1);
                    session.updated_at = now_s;
                }
                WatchSessionActionKind::RemovePlayer => {
                    let target_player_id = input.target_player_id.as_deref().ok_or_else(|| {
                        async_graphql::Error::new("targetPlayerId is required for REMOVE_PLAYER")
                    })?;
                    let target_index = session
                        .players
                        .iter()
                        .position(|player| player.id == target_player_id)
                        .ok_or_else(|| {
                            async_graphql::Error::new("Watch session player not found")
                        })?;
                    let target = session.players[target_index].clone();
                    let is_self = target.id == player.id;
                    let is_admin = auth.has_permission(UserPerms::ADMIN);
                    let is_target_inactive = is_player_inactive(&target, now_ms);
                    if !is_self && !is_admin && !is_target_inactive {
                        return Err(async_graphql::Error::new(
                            "Only admins can remove active players",
                        ));
                    }

                    session.players.remove(target_index);
                    if session.players.is_empty() {
                        return Err(async_graphql::Error::new("Watch session ended"));
                    }

                    session.revision = session.revision.saturating_add(1);
                    session.updated_at = now_s;
                }
            }

            build_beacon(session.clone(), now_ms)
        };

        let _ = runtime.sender.send(beacon.clone());
        Ok(beacon)
    }

    async fn get_or_create_runtime(
        &self,
        auth: &RequestAuth,
        input: &WatchSessionHeartbeatInput,
        now_s: i64,
    ) -> Result<(Arc<WatchSessionRuntime>, bool), async_graphql::Error> {
        if let Some(runtime) = self.get_runtime(&input.session_id).await {
            return Ok((runtime, false));
        }

        let recovery = self.resolve_recovery(auth, &input.recovery).await?;
        let runtime = Arc::new(WatchSessionRuntime::new(WatchSession {
            id: input.session_id.clone(),
            node_id: recovery.node_id,
            file_id: recovery.file_id,
            intent: recovery.intent,
            base_position_ms: recovery.base_position_ms,
            base_time_ms: recovery.base_time_ms,
            revision: 1,
            created_at: now_s,
            updated_at: now_s,
            players: Vec::new(),
        }));

        let mut runtimes = self.runtimes.lock().await;
        match runtimes.entry(input.session_id.clone()) {
            std::collections::hash_map::Entry::Occupied(entry) => Ok((entry.get().clone(), false)),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(runtime.clone());
                Ok((runtime, true))
            }
        }
    }

    async fn get_runtime(&self, session_id: &str) -> Option<Arc<WatchSessionRuntime>> {
        self.runtimes.lock().await.get(session_id).cloned()
    }

    async fn reconcile_all(&self) -> Result<(), async_graphql::Error> {
        let runtimes = self
            .runtimes
            .lock()
            .await
            .iter()
            .map(|(session_id, runtime)| (session_id.clone(), runtime.clone()))
            .collect::<Vec<_>>();

        let now_ms = current_time_ms();
        let mut sessions_to_remove = Vec::new();
        for (session_id, runtime) in runtimes {
            let beacon = {
                let session = runtime.session.lock().await;
                if session.players.is_empty() {
                    sessions_to_remove.push(session_id.clone());
                    None
                } else if all_players_inactive(&session.players, now_ms)
                    && now_ms.saturating_sub(latest_report_ms(&session.players))
                        >= INACTIVE_ONLY_DELETE_AFTER_MS
                {
                    sessions_to_remove.push(session_id.clone());
                    None
                } else if session.players.len() > 1 {
                    Some(build_beacon(session.clone(), now_ms))
                } else {
                    None
                }
            };

            if let Some(beacon) = beacon {
                let _ = runtime.sender.send(beacon);
            }
        }

        if !sessions_to_remove.is_empty() {
            let mut runtimes = self.runtimes.lock().await;
            for session_id in sessions_to_remove {
                runtimes.remove(&session_id);
            }
        }

        Ok(())
    }

    async fn resolve_recovery(
        &self,
        auth: &RequestAuth,
        recovery: &WatchSessionRecoveryInput,
    ) -> Result<ResolvedWatchSessionRecovery, async_graphql::Error> {
        let node = self.resolve_target_node(auth, &recovery.node_id).await?;
        let file = files::Entity::find_by_id(recovery.file_id.clone())
            .one(&self.pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Playable item is unavailable"))?;
        ensure_library_access(&self.pool, auth, &file.library_id)
            .await
            .map_err(|_| async_graphql::Error::new("Playable item is unavailable"))?;

        Ok(ResolvedWatchSessionRecovery {
            node_id: node.id,
            file_id: file.id,
            intent: recovery.intent,
            base_position_ms: recovery.base_position_ms.max(0),
            base_time_ms: recovery.base_time_ms as i64,
        })
    }

    async fn ensure_session_access(
        &self,
        auth: &RequestAuth,
        session: &WatchSession,
    ) -> Result<(), async_graphql::Error> {
        let file = files::Entity::find_by_id(session.file_id.clone())
            .one(&self.pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Watch session file not found"))?;
        ensure_library_access(&self.pool, auth, &file.library_id)
            .await
            .map_err(async_graphql::Error::from)
    }

    async fn resolve_target_node(
        &self,
        auth: &RequestAuth,
        node_id: &str,
    ) -> Result<nodes::Model, async_graphql::Error> {
        let node = nodes::Entity::find_by_id(node_id)
            .one(&self.pool)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Playable item not found"))?;
        ensure_library_access(&self.pool, auth, &node.library_id)
            .await
            .map_err(|_| async_graphql::Error::new("Playable item not found"))?;
        if node.kind != nodes::NodeKind::Movie && node.kind != nodes::NodeKind::Episode {
            return Err(async_graphql::Error::new("Playable item not found"));
        }
        Ok(node)
    }

    async fn resolve_target_file(
        &self,
        node: &nodes::Model,
    ) -> Result<files::Model, async_graphql::Error> {
        NodeProperties::primary_file_for_node(&self.pool, &node.id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Playable item is unavailable"))
    }

    async fn remove_runtime(&self, session_id: &str) {
        self.runtimes.lock().await.remove(session_id);
    }
}

struct ResolvedWatchSessionRecovery {
    node_id: String,
    file_id: String,
    intent: WatchSessionIntent,
    base_position_ms: i32,
    base_time_ms: i64,
}

#[ComplexObject]
impl WatchSession {
    async fn base_time_ms(&self) -> f64 {
        self.base_time_ms as f64
    }

    async fn node(&self, ctx: &Context<'_>) -> Result<Option<nodes::Model>, async_graphql::Error> {
        Ok(nodes::Entity::find_by_id(self.node_id.clone())
            .one(ctx.data::<DatabaseConnection>()?)
            .await?)
    }

    async fn file(&self, ctx: &Context<'_>) -> Result<Option<files::Model>, async_graphql::Error> {
        Ok(files::Entity::find_by_id(self.file_id.clone())
            .one(ctx.data::<DatabaseConnection>()?)
            .await?)
    }

    async fn mode(&self) -> WatchSessionMode {
        mode_for_player_count(self.players.len())
    }

    async fn effective_state(&self) -> EffectiveWatchSessionState {
        effective_state_for(self, current_time_ms())
    }

    async fn current_position_ms(&self) -> i32 {
        current_position_ms(self, current_time_ms())
    }
}

#[ComplexObject]
impl WatchSessionPlayer {
    async fn is_buffering(&self) -> bool {
        self.is_buffering
    }

    async fn base_time_ms(&self) -> f64 {
        self.base_time_ms as f64
    }

    async fn last_report_ms(&self) -> f64 {
        self.last_report_ms as f64
    }

    async fn user(&self, ctx: &Context<'_>) -> Result<Option<users::Model>, async_graphql::Error> {
        Ok(users::Entity::find_by_id(self.user_id.clone())
            .one(ctx.data::<DatabaseConnection>()?)
            .await?)
    }

    async fn is_inactive(&self) -> bool {
        is_player_inactive(self, current_time_ms())
    }

    async fn can_remove(&self, ctx: &Context<'_>) -> Result<bool, async_graphql::Error> {
        let auth = ctx.data::<RequestAuth>()?;
        Ok(auth.has_permission(UserPerms::ADMIN) || is_player_inactive(self, current_time_ms()))
    }
}

fn build_beacon(session: WatchSession, now_ms: i64) -> WatchSessionBeacon {
    let effective_state = effective_state_for(&session, now_ms);
    let base_position_ms = current_position_ms_for_state(&session, effective_state, now_ms);

    WatchSessionBeacon {
        session_id: session.id.clone(),
        node_id: session.node_id.clone(),
        file_id: session.file_id.clone(),
        mode: mode_for_player_count(session.players.len()),
        intent: session.intent,
        effective_state,
        base_position_ms,
        // send a consistent transport anchor so clients do not double-count elapsed time.
        base_time_ms: now_ms,
        revision: session.revision,
        players: session.players,
    }
}

fn update_session_transport(
    session: &mut WatchSession,
    next_intent: Option<WatchSessionIntent>,
    position_ms: i32,
    now_ms: i64,
    now_s: i64,
) {
    if let Some(intent) = next_intent {
        session.intent = intent;
    }
    session.base_position_ms = position_ms.max(0);
    session.base_time_ms = now_ms;
    session.revision = session.revision.saturating_add(1);
    session.updated_at = now_s;
}

fn mode_for_player_count(player_count: usize) -> WatchSessionMode {
    if player_count > 1 {
        WatchSessionMode::Synced
    } else {
        WatchSessionMode::Advisory
    }
}

fn effective_state_for(session: &WatchSession, now_ms: i64) -> EffectiveWatchSessionState {
    if session
        .players
        .iter()
        .any(|player| is_player_inactive(player, now_ms))
    {
        EffectiveWatchSessionState::InactivePlayers
    } else if session.players.iter().any(|player| player.is_buffering) {
        EffectiveWatchSessionState::Buffering
    } else {
        match session.intent {
            WatchSessionIntent::Playing => EffectiveWatchSessionState::Playing,
            WatchSessionIntent::Paused => EffectiveWatchSessionState::Paused,
        }
    }
}

fn current_position_ms(session: &WatchSession, now_ms: i64) -> i32 {
    current_position_ms_for_state(session, effective_state_for(session, now_ms), now_ms)
}

fn current_position_ms_for_state(
    session: &WatchSession,
    effective_state: EffectiveWatchSessionState,
    now_ms: i64,
) -> i32 {
    match effective_state {
        EffectiveWatchSessionState::Playing => {
            let elapsed_ms = now_ms.saturating_sub(session.base_time_ms);
            let next = i64::from(session.base_position_ms).saturating_add(elapsed_ms);
            next.clamp(0, i64::from(i32::MAX)) as i32
        }
        EffectiveWatchSessionState::Paused
        | EffectiveWatchSessionState::Buffering
        | EffectiveWatchSessionState::InactivePlayers => session.base_position_ms,
    }
}

fn all_players_inactive(players: &[WatchSessionPlayer], now_ms: i64) -> bool {
    players
        .iter()
        .all(|player| is_player_inactive(player, now_ms))
}

fn latest_report_ms(players: &[WatchSessionPlayer]) -> i64 {
    players
        .iter()
        .map(|player| player.last_report_ms)
        .max()
        .unwrap_or(0)
}

fn is_player_inactive(player: &WatchSessionPlayer, now_ms: i64) -> bool {
    now_ms.saturating_sub(player.last_report_ms) > HEARTBEAT_TIMEOUT_MS
}

fn current_time_ms() -> i64 {
    Utc::now().timestamp_millis()
}
