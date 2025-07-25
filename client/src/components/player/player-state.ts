import type { FragmentOf } from "gql.tada";
import { create } from "zustand/react";
import type { PlayerFrag } from "./player-wrapper";

interface PlayerState {
	currentMedia: FragmentOf<typeof PlayerFrag> | null;
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

export const setPlayerMedia = (media: FragmentOf<typeof PlayerFrag> | null) => {
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
