import { useEffect, type FC } from "react";
import { hydratePlayerFromSnapshot, usePlayerContext } from "./player-context";
import { Player } from "./player";

export const PlayerWrapper: FC = () => {
	const currentItemId = usePlayerContext((ctx) => ctx.currentItemId);
	const snapshot = usePlayerContext((ctx) => ctx.snapshot);
	const autoplay = usePlayerContext((ctx) => ctx.state.autoplay);
	const shouldPromptResume = usePlayerContext((ctx) => ctx.state.shouldPromptResume);

	useEffect(() => {
		if (currentItemId || !snapshot?.currentItemId) return;
		hydratePlayerFromSnapshot();
	}, [currentItemId, snapshot]);

	if (!currentItemId) return null;
	return <Player itemId={currentItemId} autoplay={autoplay} shouldPromptResume={shouldPromptResume} />;
};
