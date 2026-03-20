import { create } from "zustand";
import { createJSONStorage, persist, type PersistOptions } from "zustand/middleware";

interface PlayerState {
	currentItemId: string | null;
	autoplay: boolean;
	shouldPromptResume: boolean;
	isFullscreen: boolean | null;
	volume: number;
	isMuted: boolean;
	isLoading: boolean;
}

type PersistedPlayerState = Pick<PlayerState, "currentItemId" | "volume" | "isMuted">;

const playerPersistOptions: PersistOptions<PlayerState, PersistedPlayerState> = {
	name: "lyra.player",
	storage: createJSONStorage(() => window.localStorage),
	partialize: (state) => ({
		currentItemId: state.currentItemId,
		volume: state.volume,
		isMuted: state.isMuted,
	}),
};

export const playerState = create<PlayerState>()(
	persist(
		() => ({
			currentItemId: null,
			autoplay: false,
			shouldPromptResume: false,
			isFullscreen: false,
			volume: 1,
			isMuted: false,
			isLoading: false,
		}),
		playerPersistOptions,
	),
);

export const setPlayerMedia = (itemId: string, autoplay: boolean | null) => {
	playerState.setState((prev) => ({
		currentItemId: itemId,
		autoplay: autoplay ?? prev.autoplay,
		shouldPromptResume: false,
	}));
};

export const openPlayerMedia = (itemId: string, autoplay: boolean | null) => {
	playerState.setState((prev) => ({
		currentItemId: itemId,
		autoplay: autoplay ?? prev.autoplay,
		shouldPromptResume: true,
		isFullscreen: true,
	}));
};

export const clearPlayerMedia = () => {
	playerState.setState({
		currentItemId: null,
		shouldPromptResume: false,
		isFullscreen: false,
	});
};

export const togglePlayerFullscreen = (isFullscreen?: boolean) => {
	playerState.setState((prev) => ({
		isFullscreen: isFullscreen ?? !prev.isFullscreen,
	}));
};

export const setPlayerVolume = (volume: number) => {
	playerState.setState({
		volume,
	});
};

export const setPlayerMuted = (isMuted: boolean) => {
	playerState.setState({
		isMuted,
	});
};

export const setPlayerLoading = (isLoading: boolean) => {
	playerState.setState({
		isLoading,
	});
};
