import { usePlayerContext } from "../player-context";

export const usePlayerActions = () => {
	return usePlayerContext((ctx) => ctx.actions);
};
