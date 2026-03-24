import { useEffect } from "react";
import { useStore } from "zustand/react";
import { usePlayerContext } from "../player-context";
import { playerState, togglePlayerFullscreen } from "../player-state";

export const useFullscreen = () => {
	const { containerRef } = usePlayerContext();
	const isFullscreen = useStore(playerState, (s) => s.isFullscreen);

	useEffect(() => {
		if (isFullscreen == null || !containerRef.current) return;
		if (isFullscreen) {
			containerRef.current.requestFullscreen({ navigationUI: "hide" }).catch(() => false);
		} else {
			document.exitFullscreen().catch(() => false);
		}
	}, [isFullscreen, containerRef]);

	// closing browser fullscreen should also update player state
	useEffect(() => {
		const handleFullscreenChange = () => {
			if (!document.fullscreenElement) {
				togglePlayerFullscreen(false);
			}
		};
		document.addEventListener("fullscreenchange", handleFullscreenChange);
		return () => {
			document.removeEventListener("fullscreenchange", handleFullscreenChange);
		};
	}, []);
};
