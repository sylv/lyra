import { useEffect, useRef } from "react";
import { togglePlayerFullscreen } from "../player-state";

interface SurfaceInteractionOptions {
	togglePlaying: () => void;
	showControlsTemporarily: () => void;
}

export const useSurfaceInteraction = ({ togglePlaying, showControlsTemporarily }: SurfaceInteractionOptions) => {
	const doubleClickTimeoutRef = useRef<NodeJS.Timeout | null>(null);

	useEffect(() => {
		return () => {
			if (doubleClickTimeoutRef.current) {
				clearTimeout(doubleClickTimeoutRef.current);
				doubleClickTimeoutRef.current = null;
			}
		};
	}, []);

	const handleMouseMove = () => {
		showControlsTemporarily();
	};

	// on double click toggle fullscreen; on single click play/pause.
	// the timeout prevents pausing on the first click of a double click.
	const handleContainerClick = () => {
		if (doubleClickTimeoutRef.current != null) {
			clearTimeout(doubleClickTimeoutRef.current);
			doubleClickTimeoutRef.current = null;
			togglePlayerFullscreen();
			showControlsTemporarily();
			return;
		}

		doubleClickTimeoutRef.current = setTimeout(() => {
			togglePlaying();
			showControlsTemporarily();
			doubleClickTimeoutRef.current = null;
		}, 300);
	};

	return { handleMouseMove, handleContainerClick };
};
