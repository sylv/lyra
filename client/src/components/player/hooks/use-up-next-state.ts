import { useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "zustand/react";
import { playerState } from "../player-state";
import { videoState } from "../video-state";

// show the up-next card in the last 30s or 10% of the video, whichever is shorter
const PREVIEW_WINDOW_SECONDS = 30;
const PREVIEW_WINDOW_FRACTION = 0.1;
// extra countdown time after the video ends before auto-advancing
export const POST_END_COUNTDOWN_SECONDS = 10;
const TICK_INTERVAL_MS = 100;

export const useUpNextState = ({ hasNextItem, onNextItem }: { hasNextItem: boolean; onNextItem: () => void }) => {
	const currentTime = useStore(videoState, (s) => s.currentTime);
	const duration = useStore(videoState, (s) => s.duration);
	const ended = useStore(videoState, (s) => s.ended);
	const upNextDismissed = useStore(videoState, (s) => s.upNextDismissed);
	const upNextCountdownCancelled = useStore(videoState, (s) => s.upNextCountdownCancelled);
	const { autoplayNext } = useStore(playerState);

	const previewWindowSeconds =
		duration > 0 ? Math.min(PREVIEW_WINDOW_SECONDS, duration * PREVIEW_WINDOW_FRACTION) : PREVIEW_WINDOW_SECONDS;
	const isNearEnd = duration > 0 && duration - currentTime <= previewWindowSeconds;
	const isUpNextActive = hasNextItem && !upNextDismissed && (isNearEnd || ended);

	// reset dismissal/cancel state when seeking out of the preview window
	const wasActiveRef = useRef(false);
	useEffect(() => {
		if (!isUpNextActive && wasActiveRef.current) {
			videoState.setState({ upNextDismissed: false, upNextCountdownCancelled: false });
		}
		wasActiveRef.current = isUpNextActive;
	}, [isUpNextActive]);

	// sync isUpNextActive into videoState so other hooks can reference it
	useEffect(() => {
		videoState.setState({ isUpNextActive });
	}, [isUpNextActive]);

	const totalCountdownSeconds = previewWindowSeconds + POST_END_COUNTDOWN_SECONDS;
	const previewStartTime = duration - previewWindowSeconds;

	const [elapsedSinceEnd, setElapsedSinceEnd] = useState(0);
	const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

	const shouldCountdown = ended && autoplayNext && !upNextCountdownCancelled && isUpNextActive;

	useEffect(() => {
		if (!shouldCountdown) {
			setElapsedSinceEnd(0);
			if (intervalRef.current) {
				clearInterval(intervalRef.current);
				intervalRef.current = null;
			}
			return;
		}

		intervalRef.current = setInterval(() => {
			setElapsedSinceEnd((prev) => prev + TICK_INTERVAL_MS);
		}, TICK_INTERVAL_MS);

		return () => {
			if (intervalRef.current) {
				clearInterval(intervalRef.current);
				intervalRef.current = null;
			}
		};
	}, [shouldCountdown]);

	// auto-advance when countdown completes
	useEffect(() => {
		if (shouldCountdown && elapsedSinceEnd >= POST_END_COUNTDOWN_SECONDS * 1000) {
			onNextItem();
		}
	}, [shouldCountdown, elapsedSinceEnd, onNextItem]);

	// progress: fills smoothly from card appearance through to post-end countdown
	const upNextProgress = useMemo(() => {
		if (!isUpNextActive || !autoplayNext || upNextCountdownCancelled) return 0;
		if (totalCountdownSeconds <= 0) return 0;

		if (ended) {
			// video is over — progress continues from where playback left off
			const playbackPortion = previewWindowSeconds / totalCountdownSeconds;
			const postEndPortion = elapsedSinceEnd / 1000 / totalCountdownSeconds;
			return Math.min(1, playbackPortion + postEndPortion);
		}

		// still playing near end — progress based on video position
		return Math.min(1, Math.max(0, (currentTime - previewStartTime) / totalCountdownSeconds));
	}, [
		isUpNextActive,
		autoplayNext,
		upNextCountdownCancelled,
		ended,
		currentTime,
		previewStartTime,
		previewWindowSeconds,
		totalCountdownSeconds,
		elapsedSinceEnd,
	]);

	// seconds remaining until auto-advance — used for the button label
	const countdownSeconds = useMemo(() => {
		if (!isUpNextActive || !autoplayNext || upNextCountdownCancelled) return 0;
		if (ended) return Math.max(0, POST_END_COUNTDOWN_SECONDS - elapsedSinceEnd / 1000);
		return Math.max(0, duration - currentTime + POST_END_COUNTDOWN_SECONDS);
	}, [isUpNextActive, autoplayNext, upNextCountdownCancelled, ended, elapsedSinceEnd, duration, currentTime]);

	// show action buttons (Play Now / Cancel) only when active
	const showActions = isUpNextActive && (isNearEnd || ended);

	return {
		isUpNextActive,
		showActions,
		upNextProgress,
		countdownSeconds,
		autoplayNext,
		upNextCountdownCancelled,
	};
};
