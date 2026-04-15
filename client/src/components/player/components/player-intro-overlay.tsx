import { AnimatePresence } from "motion/react";
import { useMemo, type FC } from "react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { usePlayerContext } from "../player-context";
import { usePlayerActions } from "../hooks/use-player-actions";
import { SkipIntroButton } from "./skip-intro-button";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const PlayerIntroOverlay: FC<{ media: CurrentMedia }> = ({ media }) => {
  const currentTime = usePlayerContext((ctx) => ctx.state.currentTime);
  const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
  const { seekTo } = usePlayerActions();

  const introSegment = useMemo(() => {
    const segments = media.file?.segments;
    if (!Array.isArray(segments)) return null;
    return (
      segments.find(
        (segment) =>
          segment.kind === "INTRO" &&
          typeof segment.startMs === "number" &&
          typeof segment.endMs === "number" &&
          segment.endMs > segment.startMs,
      ) ?? null
    );
  }, [media.file?.segments]);

  const introProgressPercent = useMemo(() => {
    if (!introSegment) return 0;
    const introDurationMs = introSegment.endMs - introSegment.startMs;
    if (introDurationMs <= 0) return 0;
    const positionMs = currentTime * 1000;
    return Math.max(0, Math.min(1, (positionMs - introSegment.startMs) / introDurationMs));
  }, [currentTime, introSegment]);

  const isInsideIntroSegment = useMemo(() => {
    if (!introSegment) return false;
    const positionMs = currentTime * 1000;
    return positionMs >= introSegment.startMs && positionMs < introSegment.endMs;
  }, [currentTime, introSegment]);

  return (
    <div className="pointer-events-none absolute bottom-36 right-0 flex justify-end px-4">
      <div className="pointer-events-auto">
        <AnimatePresence>
          {introSegment && isInsideIntroSegment && isFullscreen ? (
            <SkipIntroButton
              key="skip-intro"
              progressPercent={introProgressPercent}
              onSkip={() => seekTo(introSegment.endMs / 1000)}
            />
          ) : null}
        </AnimatePresence>
      </div>
    </div>
  );
};
