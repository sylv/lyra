import { create } from "zustand/react";
import type { MediaWithFirstConnection } from "../../@generated/server";

interface PlayerState {
	currentMedia: MediaWithFirstConnection | null;
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

export const setPlayerMedia = (media: MediaWithFirstConnection | null) => {
	playerState.setState((prev) => ({
		currentMedia: media,
		isFullscreen: prev.currentMedia ? prev.isFullscreen : true,
	}));
};

export const setPlayerFullscreen = (isFullscreen: boolean) => {
	playerState.setState({
		isFullscreen,
	});
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
