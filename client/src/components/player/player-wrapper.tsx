import type { FC } from "react";
import { useStore } from "zustand";
import { Player } from "./player";
import { playerState } from "./player-state";

export const PlayerWrapper: FC = () => {
	const { currentItemId, autoplay } = useStore(playerState);

	if (!currentItemId) {
		return null;
	}

	return <Player itemId={currentItemId} autoplay={autoplay} />;
};
