import { useEffect, useRef } from "react";
import { useStore } from "zustand/react";
import { videoState } from "../video-state";

export const useControlsVisibility = () => {
	const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const isControlsPinned = useStore(
		videoState,
		(s) => s.isSettingsMenuOpen || s.isControlsInteracting || s.isItemCardOpen,
	);

	const areControlsPinned = () => {
		const { isSettingsMenuOpen, isControlsInteracting, isItemCardOpen } = videoState.getState();
		return isSettingsMenuOpen || isControlsInteracting || isItemCardOpen;
	};

	const showControlsTemporarily = () => {
		videoState.setState({ showControls: true });
		if (controlsTimeoutRef.current) {
			clearTimeout(controlsTimeoutRef.current);
			controlsTimeoutRef.current = null;
		}
		if (areControlsPinned()) return;
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
		if (!areControlsPinned()) {
			videoState.setState({ showControls: false });
		}
	};

	// keep cards/settings/drag interactions pinned so the overlay can't disappear mid-action.
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
