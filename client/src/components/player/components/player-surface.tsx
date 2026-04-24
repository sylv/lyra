/* oxlint-disable jsx_a11y/prefer-tag-over-role */
import { useEffect, useRef, type FC, type ReactNode } from "react";
import { playerRuntimeStore, togglePlayerFullscreen } from "../player-runtime-store";
import { usePlayerCommands } from "../hooks/use-player-commands";
import { usePlayerVisibility } from "../player-visibility";
import { cn } from "../../../lib/utils";

const NUMBER_REGEX = /^\d$/;
const ARROW_SEEK_SECONDS = 5;
const LETTER_SEEK_SECONDS = 10;

const isEditableTarget = (event: KeyboardEvent) => {
  const target = event.target instanceof HTMLElement ? event.target : null;
  const activeElement = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const candidate = activeElement ?? target;
  if (!candidate) return false;
  if (candidate.closest("[data-slot='dialog-content'], [data-slot='dropdown-menu-content']")) return true;
  return !!candidate.closest("input, textarea, select, [contenteditable=''], [contenteditable='true'], [role='textbox']");
};

export const PlayerSurface: FC<{ fullscreen: boolean; children: ReactNode }> = ({ fullscreen, children }) => {
  const { showControls, showControlsTemporarily, hideControlsImmediately } = usePlayerVisibility();
  const commands = usePlayerCommands();
  const doubleClickTimeoutRef = useRef<number | null>(null);
  const surfaceRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    return () => {
      if (doubleClickTimeoutRef.current != null) {
        window.clearTimeout(doubleClickTimeoutRef.current);
      }
    };
  }, []);

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return;
      if (!playerRuntimeStore.getState().currentItemId) return;
      if (isEditableTarget(event)) return;

      const key = event.key.toLowerCase();
      const isNumber = NUMBER_REGEX.test(event.key);
      let triggered = true;

      if (key === "arrowleft") {
        void commands.seekBy(-ARROW_SEEK_SECONDS);
      } else if (key === "arrowright") {
        void commands.seekBy(ARROW_SEEK_SECONDS);
      } else if (key === "j") {
        void commands.seekBy(-LETTER_SEEK_SECONDS);
      } else if (key === "l") {
        void commands.seekBy(LETTER_SEEK_SECONDS);
      } else if (key === "f") {
        togglePlayerFullscreen();
      } else if (key === "m") {
        commands.toggleMute();
      } else if (key === " ") {
        void commands.togglePlaying();
      } else if (event.key === "Escape") {
        togglePlayerFullscreen(false);
      } else if (isNumber) {
        const { duration } = playerRuntimeStore.getState();
        void commands.seekTo((Number.parseInt(event.key, 10) / 10) * duration);
      } else {
        triggered = false;
      }

      if (triggered) {
        showControlsTemporarily();
        event.preventDefault();
      }
    };

    document.addEventListener("keydown", handleShortcut);
    return () => document.removeEventListener("keydown", handleShortcut);
  }, [commands, showControlsTemporarily]);

  return (
    <div
      ref={surfaceRef}
      className={cn(
        "absolute inset-0 cursor-pointer select-none outline-none focus:outline-none focus-visible:outline-none focus-visible:ring-0",
        !fullscreen && "rounded",
      )}
      role="button"
      tabIndex={0}
      onMouseMove={() => {
        showControlsTemporarily();
      }}
      onMouseLeave={() => {
        hideControlsImmediately();
      }}
      onMouseDownCapture={(event) => {
        const target = event.target as HTMLElement | null;
        if (target?.closest("button, [role='slider'], [data-slot='dialog-content'], [data-slot='dropdown-menu-content']")) return;
        surfaceRef.current?.focus();
      }}
      onKeyDown={(event) => {
        if (event.key !== "Enter" || event.defaultPrevented) return;
        if (event.target !== event.currentTarget) return;
        event.preventDefault();
        void commands.togglePlaying();
      }}
      onClick={(event) => {
        const target = event.target instanceof HTMLElement ? event.target : null;
        if (target?.closest("[data-player-interactive-root], [data-slot='dialog-content'], [data-slot='dropdown-menu-content']")) {
          return;
        }
        if (doubleClickTimeoutRef.current != null) {
          window.clearTimeout(doubleClickTimeoutRef.current);
          doubleClickTimeoutRef.current = null;
          togglePlayerFullscreen();
          showControlsTemporarily();
          return;
        }

        doubleClickTimeoutRef.current = window.setTimeout(() => {
          void commands.togglePlaying();
          showControlsTemporarily();
          doubleClickTimeoutRef.current = null;
        }, 300);
      }}
      aria-label={showControls ? "Player controls visible" : "Toggle play/pause"}
    >
      {children}
    </div>
  );
};
