import { create } from "zustand";
import { useStore } from "zustand/react";
import { createJSONStorage, persist, type PersistOptions } from "zustand/middleware";

type TrackOption = { id: number; label: string };
type HoveredCard = "previous" | "next" | null;

export interface PlayerPreferences {
	volume: number;
	isMuted: boolean;
	autoplayNext: boolean;
}

export interface PlayerState {
	autoplay: boolean;
	shouldPromptResume: boolean;
	isFullscreen: boolean;
	isLoading: boolean;
	playing: boolean;
	currentTime: number;
	duration: number;
	bufferedRanges: Array<{ start: number; end: number }>;
	videoAspectRatio: number;
	errorMessage: string | null;
	audioTrackOptions: TrackOption[];
	selectedAudioTrackId: number | null;
	subtitleTrackOptions: TrackOption[];
	selectedSubtitleTrackId: number | null;
	ended: boolean;
	upNextDismissed: boolean;
	upNextCountdownCancelled: boolean;
	isUpNextActive: boolean;
}

export interface PlayerControlsState {
	showControls: boolean;
	isSettingsMenuOpen: boolean;
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
	setSubtitleTrack: (trackId: number) => void;
	setSubtitleDisplay: (enabled: boolean) => void;
	showControlsTemporarily: () => void;
	beginControlsInteraction: () => void;
	endControlsInteraction: () => void;
}

export interface PlayerContextStore {
	currentItemId: string | null;
	preferences: PlayerPreferences;
	state: PlayerState;
	controls: PlayerControlsState;
	actions: PlayerActions;
}

type PersistedPlayerContext = Pick<PlayerContextStore, "currentItemId" | "preferences">;

const noop = () => undefined;

const initialPreferences: PlayerPreferences = {
	volume: 1,
	isMuted: false,
	autoplayNext: true,
};

const initialState: PlayerState = {
	autoplay: false,
	shouldPromptResume: false,
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
	setSubtitleDisplay: noop,
	showControlsTemporarily: noop,
	beginControlsInteraction: noop,
	endControlsInteraction: noop,
};

const playerContextPersistOptions: PersistOptions<PlayerContextStore, PersistedPlayerContext> = {
	name: "lyra.player",
	storage: createJSONStorage(() => window.localStorage),
	partialize: (context) => ({
		currentItemId: context.currentItemId,
		preferences: context.preferences,
	}),
};

export const playerContext = create<PlayerContextStore>()(
	persist(
		() => ({
			currentItemId: null,
			preferences: initialPreferences,
			state: initialState,
			controls: initialControls,
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
		state: {
			...context.state,
			autoplay: autoplay ?? context.state.autoplay,
			shouldPromptResume: false,
		},
	}));
};

export const openPlayerMedia = (itemId: string, autoplay: boolean | null) => {
	playerContext.setState((context) => ({
		...context,
		currentItemId: itemId,
		state: {
			...context.state,
			autoplay: autoplay ?? context.state.autoplay,
			shouldPromptResume: true,
			isFullscreen: true,
		},
	}));
};

export const clearPlayerMedia = () => {
	playerContext.setState((context) => ({
		...context,
		currentItemId: null,
		state: {
			...context.state,
			shouldPromptResume: false,
			isFullscreen: false,
		},
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
