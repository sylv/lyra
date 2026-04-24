import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useState, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import { usePlayerCommands } from "../hooks/use-player-commands";
import { usePlayerDisplayState } from "../hooks/use-player-display-state";
import { usePlayerOptionsStore } from "../player-options-store";
import { usePlayerRuntimeStore } from "../player-runtime-store";
import { usePlayerSession } from "../player-session";
import { PlayerItemCard } from "./player-item-card";

const PREVIEW_WINDOW_SECONDS = 30;
const PREVIEW_WINDOW_FRACTION = 0.1;
const POST_END_COUNTDOWN_SECONDS = 10;
const TICK_INTERVAL_MS = 100;

export const PlayerUpNextFragment = graphql(`
  fragment PlayerUpNext on Node {
    id
    defaultFile {
      probe {
        runtimeMinutes
      }
    }
    watchProgress {
      id
      progressPercent
      completed
    }
    nextPlayable {
      id
      ...PlayerItemCard
    }
  }
`);

export const PlayerUpNextOverlay: FC<{ media: FragmentType<typeof PlayerUpNextFragment> }> = ({ media: mediaRaw }) => {
  const media = unmask(PlayerUpNextFragment, mediaRaw);
  const { switchItem } = usePlayerCommands();
  const { session } = usePlayerSession();
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);
  const ended = usePlayerRuntimeStore((state) => state.ended);
  const autoplayNext = usePlayerOptionsStore((state) => state.autoplayNext);
  const { currentTime, duration } = usePlayerDisplayState(media);
  const [dismissed, setDismissed] = useState(false);
  const [countdownCancelled, setCountdownCancelled] = useState(false);
  const [elapsedSinceEnd, setElapsedSinceEnd] = useState(0);

  useEffect(() => {
    setDismissed(false);
    setCountdownCancelled(false);
    setElapsedSinceEnd(0);
  }, [media.id]);

  const autoplayAllowed = autoplayNext && session.mode !== "SYNCED";
  const previewWindowSeconds = duration > 0 ? Math.min(PREVIEW_WINDOW_SECONDS, duration * PREVIEW_WINDOW_FRACTION) : PREVIEW_WINDOW_SECONDS;
  const isNearEnd = duration > 0 && duration - currentTime <= previewWindowSeconds;
  const isUpNextActive = isFullscreen && !!media.nextPlayable && !dismissed && (isNearEnd || ended);
  const shouldCountdown = ended && autoplayAllowed && isUpNextActive && !countdownCancelled;

  useEffect(() => {
    if (!shouldCountdown) {
      setElapsedSinceEnd(0);
      return;
    }
    const interval = window.setInterval(() => {
      setElapsedSinceEnd((value) => value + TICK_INTERVAL_MS);
    }, TICK_INTERVAL_MS);
    return () => window.clearInterval(interval);
  }, [shouldCountdown]);

  useEffect(() => {
    if (!shouldCountdown || !media.nextPlayable) return;
    if (elapsedSinceEnd < POST_END_COUNTDOWN_SECONDS * 1000) return;
    void switchItem(media.nextPlayable.id);
  }, [elapsedSinceEnd, media.nextPlayable, shouldCountdown, switchItem]);

  const totalCountdownSeconds = previewWindowSeconds + POST_END_COUNTDOWN_SECONDS;
  const previewStartTime = duration - previewWindowSeconds;
  const upNextProgress = useMemo(() => {
    if (!isUpNextActive || !autoplayAllowed || countdownCancelled) return 0;
    if (ended) {
      const playbackPortion = previewWindowSeconds / totalCountdownSeconds;
      const postEndPortion = elapsedSinceEnd / 1000 / totalCountdownSeconds;
      return Math.min(1, playbackPortion + postEndPortion);
    }
    return Math.min(1, Math.max(0, (currentTime - previewStartTime) / totalCountdownSeconds));
  }, [autoplayAllowed, countdownCancelled, currentTime, elapsedSinceEnd, ended, isUpNextActive, previewStartTime, previewWindowSeconds, totalCountdownSeconds]);

  const countdownSeconds = useMemo(() => {
    if (!isUpNextActive || !autoplayAllowed || countdownCancelled) return 0;
    if (ended) return Math.max(0, POST_END_COUNTDOWN_SECONDS - elapsedSinceEnd / 1000);
    return Math.max(0, duration - currentTime + POST_END_COUNTDOWN_SECONDS);
  }, [autoplayAllowed, countdownCancelled, currentTime, duration, elapsedSinceEnd, ended, isUpNextActive]);

  if (!media.nextPlayable || !isUpNextActive) return null;

  return (
    <div className="pointer-events-none absolute bottom-30 left-3">
      <AnimatePresence mode="wait">
        <motion.div
          key="up-next-card"
          initial={{ opacity: 0, translateX: -12 }}
          animate={{ opacity: 1, translateX: 0 }}
          exit={{ opacity: 0, translateX: -12 }}
          transition={{ duration: 0.1 }}
          className="pointer-events-auto"
        >
          <PlayerItemCard
            item={media.nextPlayable}
            onPlay={() => void switchItem(media.nextPlayable!.id)}
            onCancel={() => {
              setDismissed(true);
              setCountdownCancelled(true);
            }}
            progressPercent={upNextProgress}
            countdownSeconds={countdownSeconds}
          />
        </motion.div>
      </AnimatePresence>
    </div>
  );
};
