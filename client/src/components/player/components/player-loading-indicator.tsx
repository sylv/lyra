import { Loader2 } from "lucide-react";
import type { FC } from "react";
import { useStore } from "zustand/react";
import { playerState } from "../player-state";

export const PlayerLoadingIndicator: FC = () => {
	const isLoading = useStore(playerState, (s) => s.isLoading);
	if (!isLoading) return null;
	return (
		<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
			<Loader2 className="size-12 text-white animate-spin" />
		</div>
	);
};
