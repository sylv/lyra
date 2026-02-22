import { create } from "zustand/react";

export interface PlayerMedia {
	itemId: string;
	path: string;
}

interface PlayerState {
	currentMedia: PlayerMedia | null;
	isFullscreen: boolean | null;
	volume: number;
	isMuted: boolean;
	isLoading: boolean;
}

export const playerState = create<PlayerState>(() => ({
	currentMedia: null,
	isFullscreen: false,
	volume: 1,
	isMuted: false,
	isLoading: false,
}));

export const setPlayerMedia = (media: PlayerMedia | null) => {
	playerState.setState((prev) => ({
		currentMedia: media,
		isFullscreen: prev.currentMedia ? prev.isFullscreen : true,
	}));
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
