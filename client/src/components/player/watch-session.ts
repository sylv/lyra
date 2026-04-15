import type { WatchSessionBeaconFragmentFragment } from "../../@generated/gql/graphql";
import {
  playerContext,
  resetPlayerWatchSession,
  setPlayerState,
  setPlayerWatchSession,
  type PlayerWatchSessionPlayer,
} from "./player-context";

const normalizePlayers = (players: WatchSessionBeaconFragmentFragment["players"]): PlayerWatchSessionPlayer[] =>
  players.map((player) => ({
    id: player.id,
    userId: player.userId,
    username: player.user?.username ?? "Unknown user",
    isBuffering: player.isBuffering,
    isInactive: player.isInactive,
    canRemove: player.canRemove,
  }));

export const setPendingWatchSession = (sessionId: string | null, nodeId: string | null) => {
  setPlayerWatchSession({
    pendingSessionId: sessionId,
    pendingNodeId: nodeId,
  });
};

export const applyWatchSessionBeacon = (beacon: WatchSessionBeaconFragmentFragment) => {
  setPlayerWatchSession({
    sessionId: beacon.sessionId,
    nodeId: beacon.nodeId,
    fileId: beacon.fileId,
    mode: beacon.mode,
    intent: beacon.intent,
    effectiveState: beacon.effectiveState,
    revision: beacon.revision,
    basePositionMs: beacon.basePositionMs,
    baseTimeMs: beacon.baseTimeMs,
    players: normalizePlayers(beacon.players),
    lastContactAt: Date.now(),
    connectionWarning: null,
  });
};

export const clearWatchSession = () => {
  resetPlayerWatchSession();
  setPlayerState({ autoplay: false });
};

export const createLocalWatchSessionId = () =>
  `ws_${Math.random().toString(36).slice(2, 10)}_${Date.now().toString(36)}`;

export const getWatchSessionState = () => playerContext.getState().watchSession;

export const computeWatchSessionTargetSeconds = (beacon: WatchSessionBeaconFragmentFragment, now = Date.now()) => {
  if (beacon.effectiveState !== "PLAYING") {
    return Math.max(0, beacon.basePositionMs / 1000);
  }

  const elapsedMs = Math.max(0, now - beacon.baseTimeMs);
  return Math.max(0, (beacon.basePositionMs + elapsedMs) / 1000);
};

export const isSyncedWatchSessionActive = () => playerContext.getState().watchSession.mode === "SYNCED";
