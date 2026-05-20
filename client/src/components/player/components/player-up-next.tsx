import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useState, type FC } from "react";
import { PlayerState, playNode, usePlayerStore } from "../store/player-store";
import { PlayerItemCard } from "./player-item-card";

const PREVIEW_WINDOW_SECONDS = 30;
const PREVIEW_WINDOW_FRACTION = 0.1;
const POST_END_COUNTDOWN_SECONDS = 10;
const TICK_INTERVAL_MS = 100;

export const PlayerUpNext: FC = () => {
  const status = usePlayerStore((state) => state.status);
  const isFullscreen = usePlayerStore((state) => state.isFullscreen);
  const ended = usePlayerStore((state) => state.ended);
  const currentTime = usePlayerStore((state) => state.currentTime);
  const duration = usePlayerStore((state) => state.durationSeconds);
  const [dismissed, setDismissed] = useState(false);
  const [countdownCancelled, setCountdownCancelled] = useState(false);
  const [elapsedSinceEnd, setElapsedSinceEnd] = useState(0);
  const media = status.state === PlayerState.Mounted ? status.data.node : null;
  const nextPlayable = media?.nextPlayable ?? null;

  useEffect(() => {
    setDismissed(false);
    setCountdownCancelled(false);
    setElapsedSinceEnd(0);
  }, [media?.id]);

  const previewWindowSeconds =
    duration > 0 ? Math.min(PREVIEW_WINDOW_SECONDS, duration * PREVIEW_WINDOW_FRACTION) : PREVIEW_WINDOW_SECONDS;
  const isNearEnd = duration > 0 && duration - currentTime <= previewWindowSeconds;
  const isActive = isFullscreen && !!nextPlayable && !dismissed && (isNearEnd || ended);
  const shouldCountdown = ended && isActive && !countdownCancelled;

  useEffect(() => {
    if (!shouldCountdown) {
      setElapsedSinceEnd(0);
      return;
    }
    const interval = window.setInterval(
      () => setElapsedSinceEnd((value) => value + TICK_INTERVAL_MS),
      TICK_INTERVAL_MS,
    );
    return () => window.clearInterval(interval);
  }, [shouldCountdown]);

  useEffect(() => {
    if (!shouldCountdown || !nextPlayable) return;
    if (elapsedSinceEnd < POST_END_COUNTDOWN_SECONDS * 1000) return;
    playNode(nextPlayable.id, true);
  }, [elapsedSinceEnd, nextPlayable, shouldCountdown]);

  const totalCountdownSeconds = previewWindowSeconds + POST_END_COUNTDOWN_SECONDS;
  const previewStartTime = duration - previewWindowSeconds;
  const progress = useMemo(() => {
    if (!isActive || countdownCancelled) return 0;
    if (ended) {
      const playbackPortion = previewWindowSeconds / totalCountdownSeconds;
      const postEndPortion = elapsedSinceEnd / 1000 / totalCountdownSeconds;
      return Math.min(1, playbackPortion + postEndPortion);
    }
    return Math.min(1, Math.max(0, (currentTime - previewStartTime) / totalCountdownSeconds));
  }, [
    countdownCancelled,
    currentTime,
    elapsedSinceEnd,
    ended,
    isActive,
    previewStartTime,
    previewWindowSeconds,
    totalCountdownSeconds,
  ]);

  const countdownSeconds = useMemo(() => {
    if (!isActive || countdownCancelled) return 0;
    if (ended) return Math.max(0, POST_END_COUNTDOWN_SECONDS - elapsedSinceEnd / 1000);
    return Math.max(0, duration - currentTime + POST_END_COUNTDOWN_SECONDS);
  }, [countdownCancelled, currentTime, duration, elapsedSinceEnd, ended, isActive]);

  if (!nextPlayable || !isActive) return null;

  return (
    <div className="pointer-events-none absolute bottom-6 left-3">
      <AnimatePresence mode="wait">
        <motion.div
          key={nextPlayable.id}
          initial={{ opacity: 0, x: -12 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: -12 }}
          transition={{ duration: 0.12 }}
          className="pointer-events-auto"
          onClick={(event) => event.stopPropagation()}
          onDoubleClick={(event) => event.stopPropagation()}
        >
          <PlayerItemCard
            item={nextPlayable}
            onPlay={() => playNode(nextPlayable.id, true)}
            onCancel={() => {
              setDismissed(true);
              setCountdownCancelled(true);
            }}
            progressPercent={progress}
            countdownSeconds={countdownSeconds}
          />
        </motion.div>
      </AnimatePresence>
    </div>
  );
};
