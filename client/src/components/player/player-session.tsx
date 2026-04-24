import { createContext, useContext, useEffect, useMemo, useRef, useState, type FC, type PropsWithChildren } from "react";
import { useMutation, useSubscription } from "urql";
import { graphql, unmask } from "../../@generated/gql";
import {
  EffectiveWatchSessionState,
  type ItemPlaybackQuery,
  type WatchSessionBeaconFragmentFragment,
  WatchSessionActionKind,
  WatchSessionIntent,
  WatchSessionMode,
} from "../../@generated/gql/graphql";
import {
  playerRuntimeStore,
  setPendingWatchSession,
  setPlayerMedia,
  setPlayerRuntimeState,
} from "./player-runtime-store";
import { usePlayerVideoElement } from "./player-video-context";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;
type PlayerSessionPlayer = WatchSessionBeaconFragmentFragment["players"][number];

export interface PlayerSessionState {
  sessionId: string | null;
  playerId: string | null;
  nodeId: string | null;
  fileId: string | null;
  mode: WatchSessionMode | null;
  intent: WatchSessionIntent | null;
  effectiveState: EffectiveWatchSessionState | null;
  revision: number;
  basePositionMs: number | null;
  baseTimeMs: number | null;
  players: PlayerSessionPlayer[];
  lastContactAt: number | null;
  connectionWarning: string | null;
  isRegistered: boolean;
}

interface PlayerSessionContextValue {
  session: PlayerSessionState;
  sendAction: (
    kind: WatchSessionActionKind,
    fields?: Partial<{ positionMs: number; nodeId: string; targetPlayerId: string }>,
  ) => Promise<void>;
  leaveSession: () => Promise<void>;
}

const initialSessionState: PlayerSessionState = {
  sessionId: null,
  playerId: null,
  nodeId: null,
  fileId: null,
  mode: null,
  intent: null,
  effectiveState: null,
  revision: 0,
  basePositionMs: null,
  baseTimeMs: null,
  players: [],
  lastContactAt: null,
  connectionWarning: null,
  isRegistered: false,
};

const PlayerSessionContext = createContext<PlayerSessionContextValue | null>(null);

const createLocalWatchSessionId = () =>
  `ws_${Math.random().toString(36).slice(2, 10)}_${Date.now().toString(36)}`;

const WatchSessionBeaconFragment = graphql(`
  fragment WatchSessionBeaconFragment on WatchSessionBeacon {
    sessionId
    nodeId
    fileId
    mode
    intent
    effectiveState
    basePositionMs
    baseTimeMs
    revision
    players {
      id
      userId
      displayUsername
      isBuffering
      isInactive
      canRemove
    }
  }
`);

const LeaveWatchSession = graphql(`
  mutation LeaveWatchSession($sessionId: String!, $playerId: String!) {
    leaveWatchSession(sessionId: $sessionId, playerId: $playerId)
  }
`);

const WatchSessionHeartbeat = graphql(`
  mutation WatchSessionHeartbeat($input: WatchSessionHeartbeatInput!) {
    watchSessionHeartbeat(input: $input) {
      ...WatchSessionBeaconFragment
    }
  }
`);

const WatchSessionAction = graphql(`
  mutation WatchSessionAction($input: WatchSessionActionInput!) {
    watchSessionAction(input: $input) {
      ...WatchSessionBeaconFragment
    }
  }
`);

const WatchSessionBeacons = graphql(`
  subscription WatchSessionBeacons($sessionId: String!, $playerId: String!) {
    watchSessionBeacons(sessionId: $sessionId, playerId: $playerId) {
      ...WatchSessionBeaconFragment
    }
  }
`);

const resolveIntent = (video: HTMLVideoElement | null): WatchSessionIntent => {
  if (!video) return WatchSessionIntent.Paused;
  return video.paused ? WatchSessionIntent.Paused : WatchSessionIntent.Playing;
};

const resolveEffectiveState = (video: HTMLVideoElement | null): EffectiveWatchSessionState => {
  if (!video) return EffectiveWatchSessionState.Paused;
  return video.paused ? EffectiveWatchSessionState.Paused : EffectiveWatchSessionState.Playing;
};

export const PlayerSession: FC<PropsWithChildren<{ media: CurrentMedia | null }>> = ({ media, children }) => {
  const videoElement = usePlayerVideoElement();
  const [, watchSessionHeartbeat] = useMutation(WatchSessionHeartbeat);
  const [, watchSessionAction] = useMutation(WatchSessionAction);
  const [, leaveWatchSession] = useMutation(LeaveWatchSession);
  const [session, setSession] = useState<PlayerSessionState>(initialSessionState);
  const sessionRef = useRef(session);
  sessionRef.current = session;

  const [beaconResult] = useSubscription({
    query: WatchSessionBeacons,
    variables: {
      sessionId: session.sessionId ?? "",
      playerId: session.playerId ?? "",
    },
    pause: !session.sessionId || !session.playerId || !session.isRegistered,
  });

  const applyBeacon = (beaconRaw: NonNullable<(typeof beaconResult.data)["watchSessionBeacons"]>) => {
    const beacon = unmask(WatchSessionBeaconFragment, beaconRaw);
    setSession((current) => ({
      ...current,
      sessionId: beacon.sessionId,
      nodeId: beacon.nodeId,
      fileId: beacon.fileId,
      mode: beacon.mode,
      intent: beacon.intent,
      effectiveState: beacon.effectiveState,
      revision: beacon.revision,
      basePositionMs: beacon.basePositionMs,
      baseTimeMs: beacon.baseTimeMs,
      players: beacon.players,
      lastContactAt: Date.now(),
      connectionWarning: null,
      isRegistered: current.playerId != null && beacon.players.some((player) => player.id === current.playerId),
    }));
  };

  useEffect(() => {
    const beacon = beaconResult.data?.watchSessionBeacons;
    if (!beacon) return;
    applyBeacon(beacon);
  }, [beaconResult.data?.watchSessionBeacons]);

  useEffect(() => {
    if (!media?.defaultFile) {
      setSession(initialSessionState);
      return;
    }

    setSession((current) => {
      if (current.sessionId && current.playerId) return current;

      const runtime = playerRuntimeStore.getState();
      const shouldJoin =
        runtime.pendingWatchSessionId != null && runtime.pendingWatchSessionNodeId != null && runtime.pendingWatchSessionNodeId === media.id;
      const nextSessionId = shouldJoin ? runtime.pendingWatchSessionId : createLocalWatchSessionId();
      const nextPlayerId = createLocalWatchSessionId();

      if (shouldJoin) {
        setPendingWatchSession(null, null);
      }

      return {
        ...initialSessionState,
        sessionId: nextSessionId,
        playerId: nextPlayerId,
        nodeId: media.id,
        fileId: media.defaultFile.id,
        mode: WatchSessionMode.Advisory,
        intent: resolveIntent(videoElement),
        effectiveState: resolveEffectiveState(videoElement),
        basePositionMs: Math.max(0, Math.round((videoElement?.currentTime ?? 0) * 1000)),
        baseTimeMs: Date.now(),
      };
    });
  }, [media?.defaultFile, media?.defaultFile?.id, media?.id, videoElement]);

  const sendAction: PlayerSessionContextValue["sendAction"] = async (kind, fields = {}) => {
    const current = sessionRef.current;
    if (!current.sessionId || !current.playerId) return;

    const result = await watchSessionAction({
      input: {
        sessionId: current.sessionId,
        playerId: current.playerId,
        kind,
        positionMs: fields.positionMs ?? null,
        nodeId: fields.nodeId ?? null,
        targetPlayerId: fields.targetPlayerId ?? null,
      },
    });

    if (result.error) {
      if (current.mode === WatchSessionMode.Synced) {
        setSession((state) => ({
          ...state,
          connectionWarning: "Watch session connection lost",
        }));
      }
      throw result.error;
    }

    const beacon = result.data?.watchSessionAction;
    if (beacon) {
      applyBeacon(beacon);
    }
  };

  const leaveSession = async () => {
    const current = sessionRef.current;
    if (!current.sessionId || !current.playerId) return;
    await leaveWatchSession({
      sessionId: current.sessionId,
      playerId: current.playerId,
    });
  };

  useEffect(() => {
    const video = videoElement;
    if (!video || !media?.defaultFile || !session.sessionId || !session.playerId) return;

    const sendHeartbeat = () => {
      const currentRuntime = playerRuntimeStore.getState();
      const currentSession = sessionRef.current;
      if (!currentSession.sessionId || !currentSession.playerId) return;

      const basePositionMs = Math.max(0, Math.round(video.currentTime * 1000));
      const baseTimeMs = Date.now();
      const recoveryIntent =
        currentSession.intent === WatchSessionIntent.Playing
          ? WatchSessionIntent.Playing
          : currentSession.intent === WatchSessionIntent.Paused
            ? WatchSessionIntent.Paused
            : video.paused
              ? WatchSessionIntent.Paused
              : WatchSessionIntent.Playing;

      void watchSessionHeartbeat({
        input: {
          sessionId: currentSession.sessionId,
          playerId: currentSession.playerId,
          isBuffering: currentRuntime.buffering && !video.paused,
          basePositionMs,
          baseTimeMs,
          recovery: {
            nodeId: currentSession.nodeId ?? media.id,
            fileId: currentSession.fileId ?? media.defaultFile.id,
            intent: recoveryIntent,
            basePositionMs: currentSession.basePositionMs ?? basePositionMs,
            baseTimeMs: currentSession.baseTimeMs ?? baseTimeMs,
          },
        },
      })
        .then((result) => {
          if (result.error) {
            throw result.error;
          }
          const beacon = result.data?.watchSessionHeartbeat;
          if (beacon) {
            applyBeacon(beacon);
          }
        })
        .catch((error) => {
          console.error("failed to send watch session heartbeat", error);
          if (sessionRef.current.mode === WatchSessionMode.Synced) {
            setSession((state) => ({
              ...state,
              connectionWarning: "Watch session connection lost",
            }));
          }
        });
    };

    sendHeartbeat();
    const interval = window.setInterval(sendHeartbeat, 3_000);
    return () => window.clearInterval(interval);
  }, [media?.defaultFile, media?.defaultFile?.id, media?.id, session.playerId, session.sessionId, videoElement, watchSessionHeartbeat]);

  useEffect(() => {
    const video = videoElement;
    if (!video || session.mode !== WatchSessionMode.Synced) {
      if (video) {
        video.playbackRate = 1;
      }
      return;
    }

    if (session.lastContactAt == null) return;
    const interval = window.setInterval(() => {
      const stale = Date.now() - (sessionRef.current.lastContactAt ?? 0) >= 12_000;
      if (!stale) return;
      video.pause();
      setSession((state) => ({
        ...state,
        connectionWarning: "Watch session connection lost",
      }));
    }, 1_000);

    return () => window.clearInterval(interval);
  }, [session.lastContactAt, session.mode, videoElement]);

  useEffect(() => {
    const video = videoElement;
    if (!video || session.mode !== WatchSessionMode.Synced) {
      if (video) {
        video.playbackRate = 1;
      }
      return;
    }
    if (session.basePositionMs == null || session.baseTimeMs == null || !session.nodeId) return;

    const currentItemId = playerRuntimeStore.getState().currentItemId;
    if (session.nodeId !== currentItemId) {
      setPlayerRuntimeState({
        autoplay: false,
        shouldPromptResume: false,
        targetTime: session.basePositionMs / 1000,
      });
      setPlayerMedia(session.nodeId, false);
      return;
    }

    const targetSeconds =
      session.effectiveState === EffectiveWatchSessionState.Playing
        ? Math.max(0, (session.basePositionMs + Math.max(0, Date.now() - session.baseTimeMs)) / 1000)
        : Math.max(0, session.basePositionMs / 1000);
    const driftSeconds = targetSeconds - video.currentTime;

    if (Math.abs(driftSeconds) > 15) {
      video.currentTime = targetSeconds;
      video.playbackRate = 1;
    } else if (driftSeconds > 0.75) {
      video.playbackRate = driftSeconds > 5 ? 1.1 : 1.05;
    } else if (driftSeconds < -0.75) {
      video.playbackRate = driftSeconds < -5 ? 0.9 : 0.95;
    } else {
      video.playbackRate = 1;
    }

    if (session.effectiveState === EffectiveWatchSessionState.Playing) {
      video.play().catch(() => undefined);
    } else {
      video.pause();
    }
  }, [
    session.basePositionMs,
    session.baseTimeMs,
    session.effectiveState,
    session.mode,
    session.nodeId,
    videoElement,
  ]);

  useEffect(() => {
    return () => {
      void leaveSession();
    };
  }, [leaveWatchSession]);

  const value = useMemo(
    () => ({
      session,
      sendAction,
      leaveSession,
    }),
    [session],
  );

  if (!playerRuntimeStore.getState().currentItemId) {
    return null;
  }

  return <PlayerSessionContext.Provider value={value}>{children}</PlayerSessionContext.Provider>;
};

export const usePlayerSession = () => {
  const ctx = useContext(PlayerSessionContext);
  if (!ctx) {
    throw new Error("usePlayerSession must be used inside PlayerSession");
  }
  return ctx;
};
