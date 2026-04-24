import { create } from "zustand";
import { useStore } from "zustand/react";
import { playerOptionsStore } from "./player-options-store";

export interface PlayerAudioTrackOption {
  id: number;
  label: string;
  language: string | null;
}

export interface PlayerVideoRenditionOption {
  id: string;
  label: string;
  displayInfo: string;
  onDemand: boolean;
}

export interface PlayerSubtitleTrackOption {
  id: string;
  label: string;
  language: string | null;
  flags: string[];
  autoselect: boolean;
  renditionId: string;
  renditionType: "DIRECT" | "CONVERTED" | "OCR" | "GENERATED";
  displayInfo: string;
  onDemand: boolean;
}

export interface PlayerRuntimeState {
  currentItemId: string | null;
  autoplay: boolean;
  shouldPromptResume: boolean;
  targetTime: number | null;
  isFullscreen: boolean;
  currentTime: number;
  duration: number;
  aspectRatio: number;
  playing: boolean;
  buffering: boolean;
  ended: boolean;
  errorMessage: string | null;
  hasMediaLoaded: boolean;
  resumePromptPosition: number | null;
  pendingWatchSessionId: string | null;
  pendingWatchSessionNodeId: string | null;
  selectedVideoRenditionId: string | null;
  videoRenditionOptions: PlayerVideoRenditionOption[];
  selectedAudioTrackId: number | null;
  audioTrackOptions: PlayerAudioTrackOption[];
  selectedSubtitleTrackId: string | null;
  activeSubtitleTrackId: string | null;
  activeSubtitleRenditionId: string | null;
  pendingSubtitleTrackId: string | null;
  subtitleTrackOptions: PlayerSubtitleTrackOption[];
}

const initialRuntimeState: PlayerRuntimeState = {
  currentItemId: null,
  autoplay: false,
  shouldPromptResume: false,
  targetTime: null,
  isFullscreen: false,
  currentTime: 0,
  duration: 0,
  aspectRatio: 16 / 9,
  playing: false,
  buffering: false,
  ended: false,
  errorMessage: null,
  hasMediaLoaded: false,
  resumePromptPosition: null,
  pendingWatchSessionId: null,
  pendingWatchSessionNodeId: null,
  selectedVideoRenditionId: null,
  videoRenditionOptions: [],
  selectedAudioTrackId: null,
  audioTrackOptions: [],
  selectedSubtitleTrackId: null,
  activeSubtitleTrackId: null,
  activeSubtitleRenditionId: null,
  pendingSubtitleTrackId: null,
  subtitleTrackOptions: [],
};

export const playerRuntimeStore = create<PlayerRuntimeState>(() => initialRuntimeState);

export const usePlayerRuntimeStore = <T>(selector: (state: PlayerRuntimeState) => T) =>
  useStore(playerRuntimeStore, selector);

export const setPlayerRuntimeState = (patch: Partial<PlayerRuntimeState>) => {
  playerRuntimeStore.setState((state) => ({ ...state, ...patch }));
};

export const resetPlayerRuntimeState = (patch: Partial<PlayerRuntimeState> = {}) => {
  playerRuntimeStore.setState({
    ...initialRuntimeState,
    ...patch,
  });
};

export const setPlayerMedia = (itemId: string, autoplay: boolean | null) => {
  playerRuntimeStore.setState((state) => ({
    ...state,
    currentItemId: itemId,
    autoplay: autoplay ?? state.autoplay,
    shouldPromptResume: false,
    targetTime: null,
    currentTime: 0,
    duration: 0,
    aspectRatio: 16 / 9,
    playing: false,
    buffering: false,
    ended: false,
    errorMessage: null,
    hasMediaLoaded: false,
    resumePromptPosition: null,
    selectedVideoRenditionId: null,
    videoRenditionOptions: [],
    selectedAudioTrackId: null,
    audioTrackOptions: [],
    selectedSubtitleTrackId: null,
    activeSubtitleTrackId: null,
    activeSubtitleRenditionId: null,
    pendingSubtitleTrackId: null,
    subtitleTrackOptions: [],
  }));
  playerOptionsStore.getState().setSnapshot({
    currentItemId: itemId,
    position: 0,
  });
};

export const openPlayerMedia = (itemId: string, autoplay: boolean | null) => {
  playerRuntimeStore.setState((state) => ({
    ...state,
    currentItemId: itemId,
    autoplay: autoplay ?? state.autoplay,
    shouldPromptResume: true,
    targetTime: null,
    isFullscreen: true,
    currentTime: 0,
    duration: 0,
    aspectRatio: 16 / 9,
    playing: false,
    buffering: false,
    ended: false,
    errorMessage: null,
    hasMediaLoaded: false,
    resumePromptPosition: null,
    selectedVideoRenditionId: null,
    videoRenditionOptions: [],
    selectedAudioTrackId: null,
    audioTrackOptions: [],
    selectedSubtitleTrackId: null,
    activeSubtitleTrackId: null,
    activeSubtitleRenditionId: null,
    pendingSubtitleTrackId: null,
    subtitleTrackOptions: [],
  }));
  playerOptionsStore.getState().setSnapshot({
    currentItemId: itemId,
    position: 0,
  });
};

export const clearPlayerMedia = () => {
  playerRuntimeStore.setState((state) => ({
    ...state,
    currentItemId: null,
    autoplay: false,
    shouldPromptResume: false,
    targetTime: null,
    isFullscreen: false,
    currentTime: 0,
    duration: 0,
    aspectRatio: 16 / 9,
    playing: false,
    buffering: false,
    ended: false,
    errorMessage: null,
    hasMediaLoaded: false,
    resumePromptPosition: null,
    pendingWatchSessionId: null,
    pendingWatchSessionNodeId: null,
    selectedVideoRenditionId: null,
    videoRenditionOptions: [],
    selectedAudioTrackId: null,
    audioTrackOptions: [],
    selectedSubtitleTrackId: null,
    activeSubtitleTrackId: null,
    activeSubtitleRenditionId: null,
    pendingSubtitleTrackId: null,
    subtitleTrackOptions: [],
  }));
};

export const hydratePlayerFromSnapshot = () => {
  const snapshot = playerOptionsStore.getState().snapshot;
  if (!snapshot) return;

  playerRuntimeStore.setState((state) => ({
    ...state,
    currentItemId: snapshot.currentItemId,
    autoplay: false,
    shouldPromptResume: false,
    targetTime: snapshot.position,
    errorMessage: null,
    hasMediaLoaded: false,
  }));
};

export const togglePlayerFullscreen = (isFullscreen?: boolean) => {
  playerRuntimeStore.setState((state) => ({
    ...state,
    isFullscreen: isFullscreen ?? !state.isFullscreen,
  }));
};

export const setPendingWatchSession = (sessionId: string | null, nodeId: string | null) => {
  playerRuntimeStore.setState((state) => ({
    ...state,
    pendingWatchSessionId: sessionId,
    pendingWatchSessionNodeId: nodeId,
  }));
};
