import type React from "react";
import { togglePlayerFullscreen } from "../player-state";
import { videoState } from "../video-state";
import type { usePlayerActions } from "./use-player-actions";

const NUMBER_REGEX = /^\d$/;
const ARROW_SEEK_SECONDS = 5;
const LETTER_SEEK_SECONDS = 10;

interface KeyboardShortcutsOptions {
	actions: ReturnType<typeof usePlayerActions>;
	showControlsTemporarily: () => void;
	handleContainerClick: () => void;
}

// keep player shortcuts scoped to the focused player surface so the rest of the app keeps normal typing behavior.
export const useKeyboardShortcuts = ({ actions, showControlsTemporarily, handleContainerClick }: KeyboardShortcutsOptions) => {
	const handlePlayerKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
		if (event.defaultPrevented || videoState.getState().isSettingsMenuOpen) return;
		if (event.altKey || event.ctrlKey || event.metaKey) return;

		const target = event.target as HTMLElement | null;
		if (target?.closest("[data-slot='dialog-content']") || target?.closest("[data-slot='dropdown-menu-content']")) {
			return;
		}

		if (
			target instanceof HTMLInputElement ||
			target instanceof HTMLTextAreaElement ||
			target instanceof HTMLSelectElement ||
			target?.isContentEditable
		) {
			return;
		}

		if ((event.key === " " || event.key === "Enter") && target instanceof HTMLButtonElement) {
			return;
		}

		if (event.key === "Enter" && target === event.currentTarget) {
			event.preventDefault();
			event.stopPropagation();
			handleContainerClick();
			return;
		}

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
			actions.onToggleMute();
		} else if (key === " ") {
			actions.togglePlaying();
		} else if (event.key === "Escape") {
			const { ended, upNextDismissed, upNextCountdownCancelled } = videoState.getState();
			if (ended && !upNextDismissed) {
				if (!upNextCountdownCancelled) {
					videoState.setState({ upNextCountdownCancelled: true });
				} else {
					videoState.setState({ upNextDismissed: true });
				}
			} else {
				togglePlayerFullscreen(false);
			}
		} else if (isNumber) {
			const { duration } = videoState.getState();
			const seekTo = (Number.parseInt(event.key, 10) / 10) * duration;
			if (seekTo) {
				actions.onSeek(seekTo);
			}
		} else {
			triggered = false;
		}

		if (triggered) {
			showControlsTemporarily();
			event.preventDefault();
			event.stopPropagation();
		}
	};

	return { handlePlayerKeyDown };
};
