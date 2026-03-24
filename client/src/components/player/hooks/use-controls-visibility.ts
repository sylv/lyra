import { useEffect, useRef } from "react";
import { useStore } from "zustand/react";
import { videoState } from "../video-state";

export const useControlsVisibility = () => {
	const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const isControlsPinned = useStore(videoState, (s) => s.isSettingsMenuOpen || s.isControlsInteracting);

	const showControlsTemporarily = () => {
		videoState.setState({ showControls: true });
		if (controlsTimeoutRef.current) {
			clearTimeout(controlsTimeoutRef.current);
			controlsTimeoutRef.current = null;
		}
		const { isSettingsMenuOpen, isControlsInteracting } = videoState.getState();
		if (isSettingsMenuOpen || isControlsInteracting) return;
		controlsTimeoutRef.current = setTimeout(() => {
			videoState.setState({ showControls: false });
		}, 3000);
	};

	// keep controls visible while drag interactions (seek scrubbing, volume drag) are active.
	const beginControlsInteraction = () => {
		videoState.setState({ isControlsInteracting: true, showControls: true });
		if (controlsTimeoutRef.current) {
			clearTimeout(controlsTimeoutRef.current);
			controlsTimeoutRef.current = null;
		}
	};

	const endControlsInteraction = () => {
		videoState.setState({ isControlsInteracting: false });
		showControlsTemporarily();
	};

	const handleMouseLeave = () => {
		const { isSettingsMenuOpen, isControlsInteracting } = videoState.getState();
		if (!isSettingsMenuOpen && !isControlsInteracting) {
			videoState.setState({ showControls: false });
		}
	};

	// whenever controls become pinned (settings open or drag in progress), cancel any pending hide timeout.
	useEffect(() => {
		if (!isControlsPinned) return;
		videoState.setState({ showControls: true });
		if (controlsTimeoutRef.current) {
			clearTimeout(controlsTimeoutRef.current);
			controlsTimeoutRef.current = null;
		}
	}, [isControlsPinned]);

	useEffect(() => {
		return () => {
			if (controlsTimeoutRef.current) {
				clearTimeout(controlsTimeoutRef.current);
				controlsTimeoutRef.current = null;
			}
		};
	}, []);

	return { showControlsTemporarily, beginControlsInteraction, endControlsInteraction, handleMouseLeave };
};
