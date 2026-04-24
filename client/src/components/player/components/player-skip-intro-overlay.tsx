import { AnimatePresence } from "motion/react";
import { useMemo, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import { usePlayerCommands } from "../hooks/use-player-commands";
import { usePlayerDisplayState } from "../hooks/use-player-display-state";
import { usePlayerRuntimeStore } from "../player-runtime-store";
import { SkipIntroButton } from "./skip-intro-button";

export const PlayerSkipIntroFragment = graphql(`
  fragment PlayerSkipIntro on Node {
    id
    defaultFile {
      segments {
        kind
        startMs
        endMs
      }
      probe {
        runtimeMinutes
      }
    }
    watchProgress {
      id
      progressPercent
      completed
    }
  }
`);

export const PlayerSkipIntroOverlay: FC<{ media: FragmentType<typeof PlayerSkipIntroFragment> }> = ({ media: mediaRaw }) => {
  const media = unmask(PlayerSkipIntroFragment, mediaRaw);
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);
  const { currentTime } = usePlayerDisplayState(media);
  const { seekTo } = usePlayerCommands();

  const introSegment = useMemo(() => {
    const segments = media.defaultFile?.segments;
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
  }, [media.defaultFile?.segments]);

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
    <div className="pointer-events-none absolute bottom-30 right-0 flex justify-end px-3">
      <div className="pointer-events-auto">
        <AnimatePresence>
          {introSegment && isInsideIntroSegment && isFullscreen ? (
            <SkipIntroButton key="skip-intro" progressPercent={introProgressPercent} onSkip={() => void seekTo(introSegment.endMs / 1000)} />
          ) : null}
        </AnimatePresence>
      </div>
    </div>
  );
};
