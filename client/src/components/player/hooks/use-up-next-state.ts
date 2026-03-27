import { useEffect, useMemo, useRef, useState } from "react";
import { setPlayerState, usePlayerContext } from "../player-context";

const PREVIEW_WINDOW_SECONDS = 30;
const PREVIEW_WINDOW_FRACTION = 0.1;
export const POST_END_COUNTDOWN_SECONDS = 10;
const TICK_INTERVAL_MS = 100;

export const useUpNextState = ({ hasNextItem, onNextItem }: { hasNextItem: boolean; onNextItem: () => void }) => {
	const currentTime = usePlayerContext((ctx) => ctx.state.currentTime);
	const duration = usePlayerContext((ctx) => ctx.state.duration);
	const ended = usePlayerContext((ctx) => ctx.state.ended);
	const upNextDismissed = usePlayerContext((ctx) => ctx.state.upNextDismissed);
	const upNextCountdownCancelled = usePlayerContext((ctx) => ctx.state.upNextCountdownCancelled);
	const autoplayNext = usePlayerContext((ctx) => ctx.preferences.autoplayNext);
	const watchSessionMode = usePlayerContext((ctx) => ctx.watchSession.mode);
	const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
	const autoplayAllowed = autoplayNext && watchSessionMode !== "SYNCED";

	const previewWindowSeconds =
		duration > 0 ? Math.min(PREVIEW_WINDOW_SECONDS, duration * PREVIEW_WINDOW_FRACTION) : PREVIEW_WINDOW_SECONDS;
	const isNearEnd = duration > 0 && duration - currentTime <= previewWindowSeconds;
	const isUpNextActive = isFullscreen && hasNextItem && !upNextDismissed && (isNearEnd || ended);
	const shouldCountdown = ended && isFullscreen && autoplayAllowed && !upNextCountdownCancelled && isUpNextActive;

	const wasActiveRef = useRef(false);
	useEffect(() => {
		if (!isUpNextActive && wasActiveRef.current) {
			setPlayerState({ upNextDismissed: false, upNextCountdownCancelled: false });
		}
		wasActiveRef.current = isUpNextActive;
	}, [isUpNextActive]);

	useEffect(() => {
		setPlayerState({ isUpNextActive });
	}, [isUpNextActive]);

	const totalCountdownSeconds = previewWindowSeconds + POST_END_COUNTDOWN_SECONDS;
	const previewStartTime = duration - previewWindowSeconds;
	const [elapsedSinceEnd, setElapsedSinceEnd] = useState(0);
	const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

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

	useEffect(() => {
		if (shouldCountdown && elapsedSinceEnd >= POST_END_COUNTDOWN_SECONDS * 1000) {
			onNextItem();
		}
	}, [elapsedSinceEnd, onNextItem, shouldCountdown]);

	const upNextProgress = useMemo(() => {
		if (!isUpNextActive || !autoplayAllowed || upNextCountdownCancelled) return 0;
		if (totalCountdownSeconds <= 0) return 0;
		if (ended) {
			const playbackPortion = previewWindowSeconds / totalCountdownSeconds;
			const postEndPortion = elapsedSinceEnd / 1000 / totalCountdownSeconds;
			return Math.min(1, playbackPortion + postEndPortion);
		}
		return Math.min(1, Math.max(0, (currentTime - previewStartTime) / totalCountdownSeconds));
	}, [
		autoplayAllowed,
		currentTime,
		elapsedSinceEnd,
		ended,
		isUpNextActive,
		previewStartTime,
		previewWindowSeconds,
		totalCountdownSeconds,
		upNextCountdownCancelled,
	]);

	const countdownSeconds = useMemo(() => {
		if (!isUpNextActive || !autoplayAllowed || upNextCountdownCancelled) return 0;
		if (ended) return Math.max(0, POST_END_COUNTDOWN_SECONDS - elapsedSinceEnd / 1000);
		return Math.max(0, duration - currentTime + POST_END_COUNTDOWN_SECONDS);
	}, [autoplayAllowed, currentTime, duration, elapsedSinceEnd, ended, isUpNextActive, upNextCountdownCancelled]);

	return {
		isUpNextActive,
		showActions: isUpNextActive,
		upNextProgress,
		countdownSeconds,
		autoplayNext: autoplayAllowed,
		upNextCountdownCancelled,
	};
};
