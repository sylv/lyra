import { useEffect } from "react";
import { playerContext, setPlayerState, togglePlayerFullscreen } from "../player-context";
import type { usePlayerActions } from "./use-player-actions";

const NUMBER_REGEX = /^\d$/;
const ARROW_SEEK_SECONDS = 5;
const LETTER_SEEK_SECONDS = 10;

interface KeyboardShortcutsOptions {
	actions: ReturnType<typeof usePlayerActions>;
	handleContainerClick: () => void;
}

const isEditableTarget = (event: KeyboardEvent) => {
	const target = event.target instanceof HTMLElement ? event.target : null;
	const activeElement = document.activeElement instanceof HTMLElement ? document.activeElement : null;
	const candidate = activeElement ?? target;
	if (!candidate) return false;
	if (candidate.closest("[data-slot='dialog-content'], [data-slot='dropdown-menu-content']")) return true;
	return !!candidate.closest("input, textarea, select, [contenteditable=''], [contenteditable='true'], [role='textbox']");
};

export const useKeyboardShortcuts = ({ actions, handleContainerClick }: KeyboardShortcutsOptions) => {
	useEffect(() => {
		const handleShortcut = (event: KeyboardEvent) => {
			if (event.defaultPrevented) return;
			if (event.altKey || event.ctrlKey || event.metaKey) return;
			if (!playerContext.getState().currentItemId) return;
			if (playerContext.getState().controls.isSettingsMenuOpen) return;
			if (isEditableTarget(event)) return;

			const key = event.key.toLowerCase();
			const isNumber = NUMBER_REGEX.test(event.key);
			let triggered = true;

			if (key === "arrowleft") {
				actions.seekBy(-ARROW_SEEK_SECONDS);
			} else if (key === "arrowright") {
				actions.seekBy(ARROW_SEEK_SECONDS);
			} else if (key === "j") {
				actions.seekBy(-LETTER_SEEK_SECONDS);
			} else if (key === "l") {
				actions.seekBy(LETTER_SEEK_SECONDS);
			} else if (key === "f") {
				togglePlayerFullscreen();
			} else if (key === "m") {
				actions.toggleMute();
			} else if (key === " ") {
				actions.togglePlaying();
			} else if (event.key === "Escape") {
				const { ended, upNextDismissed, upNextCountdownCancelled } = playerContext.getState().state;
				if (ended && !upNextDismissed) {
					setPlayerState({
						upNextDismissed: true,
						upNextCountdownCancelled: upNextCountdownCancelled || ended,
					});
				} else {
					togglePlayerFullscreen(false);
				}
			} else if (isNumber) {
				const { duration } = playerContext.getState().state;
				const seekTo = (Number.parseInt(event.key, 10) / 10) * duration;
				if (seekTo) {
					actions.seekTo(seekTo);
				}
			} else {
				triggered = false;
			}

			if (triggered) {
				playerContext.getState().actions.showControlsTemporarily();
				event.preventDefault();
			}
		};

		document.addEventListener("keydown", handleShortcut);
		return () => {
			document.removeEventListener("keydown", handleShortcut);
		};
	}, [actions]);

	const handlePlayerKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
		if (event.key !== "Enter" || event.defaultPrevented) return;
		if (event.target !== event.currentTarget) return;
		event.preventDefault();
		handleContainerClick();
	};

	return { handlePlayerKeyDown };
};
