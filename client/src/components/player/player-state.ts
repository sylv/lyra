import { create } from "zustand/react";
import type { MediaWithFirstConnection } from "../../@generated/server";

interface PlayerState {
	currentMedia: MediaWithFirstConnection | null;
	isFullscreen: boolean;
}

export const playerState = create<PlayerState>(() => ({
	currentMedia: null,
	isFullscreen: false,
}));

export const setPlayerMedia = (media: MediaWithFirstConnection) => {
	playerState.setState({
		currentMedia: media,
	});
};

export const setPlayerFullscreen = (isFullscreen: boolean) => {
	playerState.setState({
		isFullscreen,
	});
};
