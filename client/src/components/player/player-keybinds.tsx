import { useEffect } from "react";
import { PlayerState, usePlayerStore } from "./store/player-store";
import { useVideoControls } from "./store/player-video-context";

const isEditableTarget = (event: KeyboardEvent) => {
  const target = event.target instanceof HTMLElement ? event.target : null;
  const activeElement = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const candidate = activeElement ?? target;
  if (!candidate) return false;
  if (candidate.closest("[data-slot='dialog-content'], [data-slot='dropdown-menu-content']")) return true;
  return !!candidate.closest(
    "input, textarea, select, [contenteditable=''], [contenteditable='true'], [role='textbox']",
  );
};

export const PlayerKeybinds = () => {
  const status = usePlayerStore((state) => state.status);
  const { togglePlaying, seek, toggleMute } = useVideoControls();

  useEffect(() => {
    if (status.state !== PlayerState.Mounted) return;

    const handler = (event: KeyboardEvent) => {
      if (isEditableTarget(event)) return;

      const { currentTime, durationSeconds } = usePlayerStore.getState();

      switch (event.key.toLowerCase()) {
        case " ":
        case "k":
          event.preventDefault();
          togglePlaying();
          break;
        case "arrowleft":
          event.preventDefault();
          seek(Math.max(0, currentTime - 5));
          break;
        case "arrowright":
          event.preventDefault();
          seek(Math.min(durationSeconds, currentTime + 5));
          break;
        case "j":
          event.preventDefault();
          seek(Math.max(0, currentTime - 10));
          break;
        case "l":
          event.preventDefault();
          seek(Math.min(durationSeconds, currentTime + 10));
          break;
        case "f":
          event.preventDefault();
          usePlayerStore.setState((state) => {
            state.isFullscreen = !state.isFullscreen;
          });
          break;
        case "m":
          event.preventDefault();
          toggleMute();
          break;
        case "0":
          event.preventDefault();
          seek(0);
          break;
        case "escape":
          if (usePlayerStore.getState().isFullscreen) {
            event.preventDefault();
            usePlayerStore.setState({ isFullscreen: false });
          }
          break;
        default:
          // 1–9 jumps to 10%–90% of the video
          if (event.key >= "1" && event.key <= "9") {
            event.preventDefault();
            seek(durationSeconds * (parseInt(event.key) / 10));
          }
          break;
      }
    };

    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [status.state, togglePlaying, seek, toggleMute]);

  return null;
};
