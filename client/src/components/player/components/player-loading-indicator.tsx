import { Loader2 } from "lucide-react";
import type { FC } from "react";
import { usePlayerContext } from "../player-context";

export const PlayerLoadingIndicator: FC = () => {
	const isLoading = usePlayerContext((ctx) => ctx.state.isLoading);
	if (!isLoading) return null;
	return (
		<div className="pointer-events-none absolute inset-0 flex items-center justify-center">
			<Loader2 className="size-12 animate-spin text-white" />
		</div>
	);
};
