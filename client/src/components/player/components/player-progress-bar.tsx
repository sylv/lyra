import { useMemo, useState, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import { formatPlayerTime } from "../../../lib/format-player-time";
import { getTimelinePreviewFrameAtMs, sortTimelinePreviewSheets } from "../../../lib/timeline-preview";
import { cn } from "../../../lib/utils";
import { usePlayerCommands } from "../hooks/use-player-commands";
import { usePlayerVideoElement } from "../player-video-context";
import { usePlayerVisibility, useShowControlsLock } from "../player-visibility";

const TIMELINE_PREVIEW_THUMBNAIL_WIDTH_PX = 380;
const TIMELINE_TIME_TOOLTIP_WIDTH_PX = 56;

export const PlayerTimelinePreviewSheetFragment = graphql(`
  fragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {
    positionMs
    endMs
    sheetIntervalMs
    sheetGapSize
    asset {
      id
      signedUrl
      width
      height
    }
  }
`);

interface HoverPreviewFrame {
  assetSignedUrl: string;
  sheetWidthPx: number;
  sheetHeightPx: number;
  frameWidthPx: number;
  frameHeightPx: number;
  offsetXPx: number;
  offsetYPx: number;
}

export const PlayerProgressBar: FC<{
  currentTime: number;
  duration: number;
  timelinePreviewSheets: FragmentType<typeof PlayerTimelinePreviewSheetFragment>[];
  compact?: boolean;
}> = ({ currentTime, duration, timelinePreviewSheets, compact = false }) => {
  const { seekTo } = usePlayerCommands();
  const { showControlsTemporarily } = usePlayerVisibility();
  const videoElement = usePlayerVideoElement();
  const [bufferedRanges, setBufferedRanges] = useState<Array<{ start: number; end: number }>>([]);
  const [hoverState, setHoverState] = useState<{ time: number; xPx: number; barWidthPx: number } | null>(null);
  const [dragging, setDragging] = useState(false);
  useShowControlsLock(hoverState != null || dragging);

  const sortedTimelinePreviewSheets = useMemo(
    () => sortTimelinePreviewSheets(timelinePreviewSheets.map((sheet) => unmask(PlayerTimelinePreviewSheetFragment, sheet))),
    [timelinePreviewSheets],
  );

  const hoverPreviewFrame: HoverPreviewFrame | null = useMemo(() => {
    if (!hoverState) return null;
    return getTimelinePreviewFrameAtMs(hoverState.time * 1000, sortedTimelinePreviewSheets);
  }, [hoverState, sortedTimelinePreviewSheets]);

  const renderedHoverPreviewFrame = useMemo(() => {
    if (!hoverPreviewFrame) return null;
    const scale = TIMELINE_PREVIEW_THUMBNAIL_WIDTH_PX / hoverPreviewFrame.frameWidthPx;
    return {
      ...hoverPreviewFrame,
      frameWidthPx: Math.max(1, TIMELINE_PREVIEW_THUMBNAIL_WIDTH_PX),
      frameHeightPx: Math.max(1, hoverPreviewFrame.frameHeightPx * scale),
      scale,
      sourceFrameWidthPx: hoverPreviewFrame.frameWidthPx,
      sourceFrameHeightPx: hoverPreviewFrame.frameHeightPx,
    };
  }, [hoverPreviewFrame]);

  const hoverMarkerPercent = useMemo(() => {
    if (!hoverState || hoverState.barWidthPx <= 0) return 0;
    return (hoverState.xPx / hoverState.barWidthPx) * 100;
  }, [hoverState]);

  const clampedHoverOverlayPercent = useMemo(() => {
    if (!hoverState || hoverState.barWidthPx <= 0) return 0;
    const overlayWidthPx = renderedHoverPreviewFrame?.frameWidthPx ?? TIMELINE_TIME_TOOLTIP_WIDTH_PX;
    const minCenterPx = overlayWidthPx / 2;
    const maxCenterPx = hoverState.barWidthPx - overlayWidthPx / 2;
    const clampedCenterPx =
      minCenterPx <= maxCenterPx
        ? Math.min(Math.max(hoverState.xPx, minCenterPx), maxCenterPx)
        : hoverState.barWidthPx / 2;
    return (clampedCenterPx / hoverState.barWidthPx) * 100;
  }, [hoverState, renderedHoverPreviewFrame]);

  const progressPercent = duration > 0 ? (currentTime / duration) * 100 : 0;

  const syncBufferedRanges = () => {
    if (!videoElement) {
      setBufferedRanges([]);
      return;
    }
    const ranges: Array<{ start: number; end: number }> = [];
    for (let index = 0; index < videoElement.buffered.length; index++) {
      ranges.push({
        start: videoElement.buffered.start(index),
        end: videoElement.buffered.end(index),
      });
    }
    setBufferedRanges(ranges);
  };

  const setTimeFromPointer = (clientX: number, currentTarget: HTMLDivElement) => {
    if (!duration) return;
    const rect = currentTarget.getBoundingClientRect();
    const hoverX = clientX - rect.left;
    const ratio = Math.max(0, Math.min(1, hoverX / rect.width));
    const nextTime = Math.max(0, Math.min(duration, ratio * duration));
    setHoverState({
      time: nextTime,
      xPx: Math.max(0, Math.min(rect.width, hoverX)),
      barWidthPx: rect.width,
    });
    return nextTime;
  };

  return (
    <div
      className={cn(compact ? "cursor-pointer py-0.5" : "cursor-pointer py-1")}
      onClick={(event) => {
        event.stopPropagation();
        showControlsTemporarily();
        const nextTime = setTimeFromPointer(event.clientX, event.currentTarget);
        if (nextTime != null) {
          void seekTo(nextTime);
        }
      }}
      onMouseMove={(event) => {
        showControlsTemporarily();
        setTimeFromPointer(event.clientX, event.currentTarget);
        syncBufferedRanges();
      }}
      onMouseLeave={() => setHoverState(null)}
      onPointerDown={(event) => {
        showControlsTemporarily();
        setDragging(true);
        setTimeFromPointer(event.clientX, event.currentTarget);
        syncBufferedRanges();
      }}
      onPointerMove={(event) => {
        if (!dragging) return;
        const nextTime = setTimeFromPointer(event.clientX, event.currentTarget);
        if (nextTime != null) {
          void seekTo(nextTime);
        }
      }}
      onPointerUp={() => setDragging(false)}
      onPointerCancel={() => setDragging(false)}
      onKeyDown={(event) => {
        if (!duration) return;
        const step = 5;
        if (event.key === "ArrowLeft") {
          event.preventDefault();
          void seekTo(Math.max(0, currentTime - step));
        } else if (event.key === "ArrowRight") {
          event.preventDefault();
          void seekTo(Math.min(duration, currentTime + step));
        } else if (event.key === "Home") {
          event.preventDefault();
          void seekTo(0);
        } else if (event.key === "End") {
          event.preventDefault();
          void seekTo(duration);
        }
      }}
      role="slider"
      tabIndex={0}
      aria-label="Seek video"
      aria-valuemin={0}
      aria-valuemax={duration || 100}
      aria-valuenow={currentTime || 0}
    >
      <div className={cn("relative rounded-md bg-white/15 transition-all", compact ? "h-1.5" : "h-1 group-hover:h-2")}>
        {bufferedRanges.map((range) => {
          if (!duration) return null;
          const startPercent = (range.start / duration) * 100;
          const widthPercent = ((range.end - range.start) / duration) * 100;
          return (
            <div
              key={`${range.start}-${range.end}`}
              className="absolute top-0 h-full bg-white/15 transition-all"
              style={{ left: `${startPercent}%`, width: `${widthPercent}%` }}
            />
          );
        })}
        <div className="h-full rounded-md bg-white/80 transition-all" style={{ width: `${progressPercent}%` }} />

        {hoverState ? (
          <>
            <div className="pointer-events-none absolute inset-y-0" style={{ left: `${hoverMarkerPercent}%` }}>
              <div className="absolute -top-1 bottom-0 z-20 w-0.5 -translate-x-1/2 bg-white/40 shadow-lg" />
            </div>
            <div className="pointer-events-none absolute inset-y-0" style={{ left: `${clampedHoverOverlayPercent}%` }}>
              {renderedHoverPreviewFrame ? (
                <div
                  className="absolute bottom-4 left-1/2 -translate-x-1/2 overflow-hidden rounded-md bg-black shadow-lg"
                  style={{
                    width: `${renderedHoverPreviewFrame.frameWidthPx}px`,
                    height: `${renderedHoverPreviewFrame.frameHeightPx}px`,
                  }}
                >
                  <div
                    style={{
                      width: `${renderedHoverPreviewFrame.sourceFrameWidthPx}px`,
                      height: `${renderedHoverPreviewFrame.sourceFrameHeightPx}px`,
                      transform: `scale(${renderedHoverPreviewFrame.scale})`,
                      transformOrigin: "top left",
                      backgroundImage: `url(${renderedHoverPreviewFrame.assetSignedUrl})`,
                      backgroundPosition: `-${renderedHoverPreviewFrame.offsetXPx}px -${renderedHoverPreviewFrame.offsetYPx}px`,
                      backgroundSize: `${renderedHoverPreviewFrame.sheetWidthPx}px ${renderedHoverPreviewFrame.sheetHeightPx}px`,
                      backgroundRepeat: "no-repeat",
                    }}
                  />
                </div>
              ) : null}
              <div
                className={cn(
                  "absolute left-1/2 -translate-x-1/2 rounded bg-black/60 px-2 py-0.5 text-sm",
                  renderedHoverPreviewFrame ? "-top-10" : "-top-8",
                )}
              >
                {formatPlayerTime(hoverState.time)}
              </div>
            </div>
          </>
        ) : null}
      </div>
    </div>
  );
};
