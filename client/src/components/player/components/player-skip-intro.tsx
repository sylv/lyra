import { AnimatePresence, motion } from "motion/react";
import { useMemo, type FC } from "react";
import { FileSegmentKind } from "../../../@generated/gql/graphql";
import { PlayerState, usePlayerStore } from "../store/player-store";
import { useVideoControls } from "../store/player-video-context";

export const PlayerSkipIntro: FC = () => {
  const status = usePlayerStore((state) => state.status);
  const isFullscreen = usePlayerStore((state) => state.isFullscreen);
  const currentTime = usePlayerStore((state) => state.currentTime);
  const { seek } = useVideoControls();
  const segments = status.state === PlayerState.Mounted ? (status.data.node.defaultFile?.segments ?? []) : [];

  const introSegment = useMemo(() => {
    return (
      segments.find(
        (segment) =>
          segment.kind === FileSegmentKind.Intro &&
          typeof segment.startMs === "number" &&
          typeof segment.endMs === "number" &&
          segment.endMs > segment.startMs,
      ) ?? null
    );
  }, [segments]);

  const introProgress = useMemo(() => {
    if (!introSegment) return 0;
    const durationMs = introSegment.endMs - introSegment.startMs;
    if (durationMs <= 0) return 0;
    return Math.max(0, Math.min(1, (currentTime * 1000 - introSegment.startMs) / durationMs));
  }, [currentTime, introSegment]);

  const isInsideIntro = useMemo(() => {
    if (!introSegment) return false;
    const positionMs = currentTime * 1000;
    return positionMs >= introSegment.startMs && positionMs < introSegment.endMs;
  }, [currentTime, introSegment]);

  const clampedPercent = Math.max(0, Math.min(100, introProgress * 100));

  return (
    <div className="pointer-events-none absolute bottom-6 right-3">
      <AnimatePresence>
        {introSegment && isInsideIntro && isFullscreen ? (
          <motion.button
            key="skip-intro"
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 8 }}
            transition={{ duration: 0.2 }}
            type="button"
            className="pointer-events-auto relative overflow-hidden rounded-md bg-white/70 px-9 py-2 text-left text-sm font-semibold text-black shadow-lg backdrop-blur-sm transition-colors hover:bg-white/50"
            onClick={(event) => {
              event.stopPropagation();
              seek(introSegment.endMs / 1000);
            }}
            onDoubleClick={(event) => event.stopPropagation()}
          >
            <div className="pointer-events-none absolute inset-0">
              <div
                className="h-full bg-white/90 transition-[width] duration-300 ease-linear"
                style={{ width: `${clampedPercent}%` }}
              />
            </div>
            <span className="relative z-10">Skip Intro</span>
          </motion.button>
        ) : null}
      </AnimatePresence>
    </div>
  );
};
