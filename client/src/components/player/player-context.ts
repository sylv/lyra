import { create } from "zustand";
import { useStore } from "zustand/react";
import { createJSONStorage, persist, type PersistOptions } from "zustand/middleware";

type AudioTrackOption = { id: number; label: string };
export type SubtitleTrackOption = {
  id: string;
  label: string;
  source: "EXTRACTED" | "CONVERTED" | "OCR" | "GENERATED";
  tags: string[];
  language: string | null;
  signedUrl: string;
};
type HoveredCard = "previous" | "next" | null;
export type PlayerWatchSessionMode = "ADVISORY" | "SYNCED" | null;
export type PlayerWatchSessionEffectiveState = "PLAYING" | "PAUSED" | "BUFFERING" | "INACTIVE_PLAYERS" | null;
export type PlayerWatchSessionIntent = "PLAYING" | "PAUSED" | null;

export interface PlayerWatchSessionPlayer {
  id: string;
  userId: string;
  username: string;
  isBuffering: boolean;
  isInactive: boolean;
  canRemove: boolean;
}

export interface PlayerWatchSessionState {
  pendingSessionId: string | null;
  pendingNodeId: string | null;
  sessionId: string | null;
  playerId: string | null;
  nodeId: string | null;
  fileId: string | null;
  mode: PlayerWatchSessionMode;
  intent: PlayerWatchSessionIntent;
  effectiveState: PlayerWatchSessionEffectiveState;
  revision: number;
  basePositionMs: number | null;
  baseTimeMs: number | null;
  players: PlayerWatchSessionPlayer[];
  lastContactAt: number | null;
  connectionWarning: string | null;
}

export interface PlayerPreferences {
  volume: number;
  isMuted: boolean;
  autoplayNext: boolean;
}

export interface PlayerSnapshot {
  currentItemId: string;
  position: number;
}

export interface PlayerState {
  autoplay: boolean;
  shouldPromptResume: boolean;
  pendingInitialPosition: number | null;
  isFullscreen: boolean;
  isLoading: boolean;
  playing: boolean;
  currentTime: number;
  duration: number;
  bufferedRanges: Array<{ start: number; end: number }>;
  videoAspectRatio: number;
  errorMessage: string | null;
  audioTrackOptions: AudioTrackOption[];
  selectedAudioTrackId: number | null;
  subtitleTrackOptions: SubtitleTrackOption[];
  selectedSubtitleTrackId: string | null;
  ended: boolean;
  upNextDismissed: boolean;
  upNextCountdownCancelled: boolean;
  isUpNextActive: boolean;
}

export interface PlayerControlsState {
  showControls: boolean;
  isSettingsMenuOpen: boolean;
  isWatchSessionMenuOpen: boolean;
  isControlsInteracting: boolean;
  isItemCardOpen: boolean;
  hoveredCard: HoveredCard;
  resumePromptPosition: number | null;
  confirmResumePrompt: (() => void) | null;
  cancelResumePrompt: (() => void) | null;
}

export interface PlayerActions {
  togglePlaying: () => void;
  seekBy: (deltaSeconds: number) => void;
  seekTo: (time: number) => void;
  toggleMute: () => void;
  setVolume: (volume: number) => void;
  setAudioTrack: (trackId: number) => void;
  setSubtitleTrack: (trackId: string | null) => void;
  showControlsTemporarily: () => void;
  beginControlsInteraction: () => void;
  endControlsInteraction: () => void;
  switchItem: (itemId: string) => void;
}

export interface PlayerContextStore {
  currentItemId: string | null;
  snapshot: PlayerSnapshot | null;
  preferences: PlayerPreferences;
  state: PlayerState;
  controls: PlayerControlsState;
  watchSession: PlayerWatchSessionState;
  actions: PlayerActions;
}

type PersistedPlayerContext = Pick<PlayerContextStore, "snapshot" | "preferences">;

const noop = () => undefined;

const initialPreferences: PlayerPreferences = {
  volume: 1,
  isMuted: false,
  autoplayNext: true,
};

const initialState: PlayerState = {
  autoplay: false,
  shouldPromptResume: false,
  pendingInitialPosition: null,
  isFullscreen: false,
  isLoading: false,
  playing: false,
  currentTime: 0,
  duration: 0,
  bufferedRanges: [],
  videoAspectRatio: 16 / 9,
  errorMessage: null,
  audioTrackOptions: [],
  selectedAudioTrackId: null,
  subtitleTrackOptions: [],
  selectedSubtitleTrackId: null,
  ended: false,
  upNextDismissed: false,
  upNextCountdownCancelled: false,
  isUpNextActive: false,
};

const initialControls: PlayerControlsState = {
  showControls: true,
  isSettingsMenuOpen: false,
  isWatchSessionMenuOpen: false,
  isControlsInteracting: false,
  isItemCardOpen: false,
  hoveredCard: null,
  resumePromptPosition: null,
  confirmResumePrompt: null,
  cancelResumePrompt: null,
};

const initialActions: PlayerActions = {
  togglePlaying: noop,
  seekBy: noop,
  seekTo: noop,
  toggleMute: noop,
  setVolume: noop,
  setAudioTrack: noop,
  setSubtitleTrack: noop,
  showControlsTemporarily: noop,
  beginControlsInteraction: noop,
  endControlsInteraction: noop,
  switchItem: noop,
};

const initialWatchSession: PlayerWatchSessionState = {
  pendingSessionId: null,
  pendingNodeId: null,
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
};

const playerContextPersistOptions: PersistOptions<PlayerContextStore, PersistedPlayerContext> = {
  name: "lyra.player",
  storage: createJSONStorage(() => window.localStorage),
  merge: (persistedState, currentState) => ({
    ...currentState,
    ...(persistedState as PersistedPlayerContext),
    currentItemId: null,
  }),
  partialize: (context) => ({
    snapshot: context.snapshot,
    preferences: context.preferences,
  }),
};

export const playerContext = create<PlayerContextStore>()(
  persist(
    () => ({
      currentItemId: null,
      snapshot: null,
      preferences: initialPreferences,
      state: initialState,
      controls: initialControls,
      watchSession: initialWatchSession,
      actions: initialActions,
    }),
    playerContextPersistOptions,
  ),
);

export const usePlayerContext = <T>(selector: (ctx: PlayerContextStore) => T) => useStore(playerContext, selector);

export const setPlayerPreferences = (preferences: Partial<PlayerPreferences>) => {
  playerContext.setState((context) => ({
    ...context,
    preferences: {
      ...context.preferences,
      ...preferences,
    },
  }));
};

export const setPlayerState = (state: Partial<PlayerState>) => {
  playerContext.setState((context) => ({
    ...context,
    state: {
      ...context.state,
      ...state,
    },
  }));
};

export const setPlayerControls = (controls: Partial<PlayerControlsState>) => {
  playerContext.setState((context) => ({
    ...context,
    controls: {
      ...context.controls,
      ...controls,
    },
  }));
};

export const setPlayerActions = (actions: Partial<PlayerActions>) => {
  playerContext.setState((context) => ({
    ...context,
    actions: {
      ...context.actions,
      ...actions,
    },
  }));
};

export const setPlayerWatchSession = (watchSession: Partial<PlayerWatchSessionState>) => {
  playerContext.setState((context) => ({
    ...context,
    watchSession: {
      ...context.watchSession,
      ...watchSession,
    },
  }));
};

export const resetPlayerWatchSession = (watchSession: Partial<PlayerWatchSessionState> = {}) => {
  playerContext.setState((context) => ({
    ...context,
    watchSession: {
      ...initialWatchSession,
      ...watchSession,
    },
  }));
};

export const resetPlayerState = (state: Partial<PlayerState> = {}) => {
  playerContext.setState((context) => ({
    ...context,
    state: {
      ...initialState,
      ...state,
    },
  }));
};

export const resetPlayerControls = (controls: Partial<PlayerControlsState> = {}) => {
  playerContext.setState((context) => ({
    ...context,
    controls: {
      ...initialControls,
      ...controls,
    },
  }));
};

export const setPlayerMedia = (itemId: string, autoplay: boolean | null) => {
  playerContext.setState((context) => ({
    ...context,
    currentItemId: itemId,
    snapshot: {
      currentItemId: itemId,
      position: 0,
    },
    state: {
      ...context.state,
      autoplay: autoplay ?? context.state.autoplay,
      shouldPromptResume: false,
      pendingInitialPosition: null,
    },
  }));
};

export const openPlayerMedia = (itemId: string, autoplay: boolean | null) => {
  playerContext.setState((context) => ({
    ...context,
    currentItemId: itemId,
    snapshot: {
      currentItemId: itemId,
      position: 0,
    },
    state: {
      ...context.state,
      autoplay: autoplay ?? context.state.autoplay,
      shouldPromptResume: true,
      pendingInitialPosition: null,
      isFullscreen: true,
    },
  }));
};

export const clearPlayerMedia = () => {
  playerContext.setState((context) => ({
    ...context,
    currentItemId: null,
    snapshot: null,
    state: {
      ...context.state,
      shouldPromptResume: false,
      pendingInitialPosition: null,
      isFullscreen: false,
    },
    watchSession: {
      ...initialWatchSession,
    },
  }));
};

export const hydratePlayerFromSnapshot = () => {
  playerContext.setState((context) => {
    if (!context.snapshot) return context;
    return {
      ...context,
      currentItemId: context.snapshot.currentItemId,
      state: {
        ...context.state,
        autoplay: false,
        shouldPromptResume: false,
        pendingInitialPosition: context.snapshot.position,
      },
    };
  });
};

export const setPlayerSnapshot = (snapshot: PlayerSnapshot | null) => {
  playerContext.setState((context) => ({
    ...context,
    snapshot,
  }));
};

export const togglePlayerFullscreen = (isFullscreen?: boolean) => {
  playerContext.setState((context) => ({
    ...context,
    state: {
      ...context.state,
      isFullscreen: isFullscreen ?? !context.state.isFullscreen,
    },
  }));
};

export const setPlayerVolume = (volume: number) => {
  setPlayerPreferences({ volume });
};

export const setPlayerMuted = (isMuted: boolean) => {
  setPlayerPreferences({ isMuted });
};

export const setPlayerLoading = (isLoading: boolean) => {
  setPlayerState({ isLoading });
};
