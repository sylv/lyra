import type { FC } from "react";
import { useStore } from "zustand";
import { Player } from "./player";
import { playerState } from "./player-state";

export const PlayerWrapper: FC = () => {
	const { currentMedia } = useStore(playerState);
	if (!currentMedia) {
		return null;
	}

	return <Player itemId={currentMedia.itemId} />;
};
