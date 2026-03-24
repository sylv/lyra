import type { FC } from "react";
import { useStore } from "zustand/react";
import { videoState } from "../video-state";

export const PlayerErrorOverlay: FC = () => {
	const errorMessage = useStore(videoState, (s) => s.errorMessage);
	if (!errorMessage) return null;
	return (
		<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
			<div className="text-white text-center p-4 mt-24 pointer-events-auto">
				<p>{errorMessage}</p>
			</div>
		</div>
	);
};
