import { useEffect } from "react";
import { togglePlayerFullscreen, usePlayerContext } from "../player-context";
import { usePlayerRefsContext } from "../player-refs-context";

export const useFullscreen = () => {
	const { containerRef } = usePlayerRefsContext();
	const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);

	useEffect(() => {
		if (!containerRef.current) return;
		if (isFullscreen) {
			containerRef.current.requestFullscreen({ navigationUI: "hide" }).catch(() => false);
		} else if (document.fullscreenElement) {
			document.exitFullscreen().catch(() => false);
		}
	}, [containerRef, isFullscreen]);

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
