import type { FC } from "react";
import { usePlayerContext } from "./player-context";
import { Player } from "./player";

export const PlayerWrapper: FC = () => {
	const currentItemId = usePlayerContext((ctx) => ctx.currentItemId);
	const autoplay = usePlayerContext((ctx) => ctx.state.autoplay);
	const shouldPromptResume = usePlayerContext((ctx) => ctx.state.shouldPromptResume);

	if (!currentItemId) return null;
	return <Player itemId={currentItemId} autoplay={autoplay} shouldPromptResume={shouldPromptResume} />;
};
