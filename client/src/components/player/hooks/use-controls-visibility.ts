import { useEffect, useRef } from "react";
import { playerContext, setPlayerActions, setPlayerControls, usePlayerContext } from "../player-context";

export const useControlsVisibility = () => {
  const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
  const pendingSubtitleTrackId = usePlayerContext((ctx) => ctx.state.pendingSubtitleTrackId);
  const errorMessage = usePlayerContext((ctx) => ctx.state.errorMessage);
  const isControlsPinned = usePlayerContext(
    (ctx) =>
      ctx.controls.isSettingsMenuOpen ||
      ctx.controls.isWatchSessionMenuOpen ||
      ctx.controls.isControlsInteracting ||
      ctx.controls.isItemCardOpen ||
      ctx.state.pendingSubtitleTrackId != null ||
      ctx.state.errorMessage != null,
  );

  const areControlsPinned = () => {
    const { controls, state } = playerContext.getState();
    return (
      controls.isSettingsMenuOpen ||
      controls.isWatchSessionMenuOpen ||
      controls.isControlsInteracting ||
      controls.isItemCardOpen ||
      state.pendingSubtitleTrackId != null ||
      state.errorMessage != null
    );
  };

  const showControlsTemporarily = () => {
    setPlayerControls({ showControls: true });
    if (controlsTimeoutRef.current) {
      clearTimeout(controlsTimeoutRef.current);
      controlsTimeoutRef.current = null;
    }
    if (areControlsPinned()) return;
    controlsTimeoutRef.current = setTimeout(() => {
      setPlayerControls({ showControls: false });
    }, 3000);
  };

  const beginControlsInteraction = () => {
    setPlayerControls({ isControlsInteracting: true, showControls: true });
    if (controlsTimeoutRef.current) {
      clearTimeout(controlsTimeoutRef.current);
      controlsTimeoutRef.current = null;
    }
  };

  const endControlsInteraction = () => {
    setPlayerControls({ isControlsInteracting: false });
    showControlsTemporarily();
  };

  const handleMouseLeave = () => {
    if (!areControlsPinned()) {
      setPlayerControls({ showControls: false });
    }
  };

  useEffect(() => {
    setPlayerActions({
      showControlsTemporarily,
      beginControlsInteraction,
      endControlsInteraction,
    });
  }, [isFullscreen, pendingSubtitleTrackId, errorMessage]);

  useEffect(() => {
    if (!isControlsPinned) return;
    setPlayerControls({ showControls: true });
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

  return { handleMouseLeave, showControlsTemporarily };
};
