// oxlint-disable jsx_a11y/click-events-have-key-events
// oxlint-disable jsx_a11y/no-static-element-interactions
import { motion } from "framer-motion";
import prettyMilliseconds from "pretty-ms";
import React, { useMemo, useRef, useState, type FC } from "react";
import { unmask } from "../../../@generated/gql";
import { getTimelinePreviewFrameAtMs, sortTimelinePreviewSheets } from "../../../lib/timeline-preview";
import { cn } from "../../../lib/utils";
import { PLAYER_GLASS_CLASS } from "../constants";
import { useControlsOverride } from "../store/player-controls-store";
import { PlayerState, PlayerTimelinePreviewSheet, usePlayerStore } from "../store/player-store";
import { useVideoControls } from "../store/player-video-context";

const formatTime = (seconds: number) => {
  return prettyMilliseconds(seconds * 1000, {
    colonNotation: true,
    secondsDecimalDigits: 0,
  });
};

const clamp = (value: number, min: number, max: number) => {
  return Math.min(Math.max(value, min), max);
};

const getPreviewLeft = ({
  desiredLeft,
  seekWidth,
  seekViewportLeft,
  seekViewportRight,
  previewWidth,
  isFullscreen,
}: {
  desiredLeft: number;
  seekWidth: number;
  seekViewportLeft: number;
  seekViewportRight: number;
  previewWidth: number;
  isFullscreen: boolean;
}) => {
  const stickyMinLeft = 0;
  const stickyMaxLeft = Math.max(0, seekWidth - previewWidth);

  if (isFullscreen) {
    return clamp(desiredLeft, stickyMinLeft, stickyMaxLeft);
  }

  const viewportWidth = window.innerWidth;
  const spaceToViewportLeft = seekViewportLeft;
  const spaceToViewportRight = viewportWidth - seekViewportRight;

  // the preview may overflow the seek bar only when there is at least one full preview width available on that side.
  const canOverflowLeft = spaceToViewportLeft >= previewWidth;
  const canOverflowRight = spaceToViewportRight >= previewWidth;
  const minLeft = canOverflowLeft ? -previewWidth / 2 : stickyMinLeft;
  const maxLeft = canOverflowRight ? seekWidth - previewWidth / 2 : stickyMaxLeft;

  return clamp(desiredLeft, minLeft, maxLeft);
};

export const PlayerSeekBar: FC = () => {
  const { seek } = useVideoControls();
  const currentTime = usePlayerStore((state) => state.currentTime);
  const durationSeconds = usePlayerStore((state) => state.durationSeconds);
  const bufferedRanges = usePlayerStore((state) => state.bufferedRanges);
  const isFullscreen = usePlayerStore((state) => state.isFullscreen);
  const status = usePlayerStore((state) => state.status);
  const remainingSeconds = Math.max(durationSeconds - currentTime, 0);
  const seekRef = useRef<HTMLDivElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);
  const [showPreview, setShowPreview] = useState(false);
  // cursor position within the seek bar
  const [cursorLeft, setCursorLeft] = useState(0);
  // how far the preview should be from the left of the seek bar viewport
  const [previewLeft, setPreviewLeft] = useState(0);
  const [previewTime, setPreviewTime] = useState(0);

  useControlsOverride(showPreview);

  const timelinePreviewSheets = useMemo(() => {
    if (status.state !== PlayerState.Mounted) return [];
    return sortTimelinePreviewSheets(
      status.data.node.defaultFile?.timelinePreview.map((sheet) => unmask(PlayerTimelinePreviewSheet, sheet)) ?? [],
    );
  }, [status]);

  const previewFrame = useMemo(() => {
    return getTimelinePreviewFrameAtMs(previewTime * 1000, timelinePreviewSheets);
  }, [previewTime, timelinePreviewSheets]);

  const handlePointerMove = (event: React.PointerEvent<HTMLDivElement>) => {
    if (!seekRef.current || !previewRef.current || !durationSeconds) return;

    const seekRect = seekRef.current.getBoundingClientRect();
    const previewWidth = previewRef.current.offsetWidth;

    const seekX = clamp(event.clientX - seekRect.left, 0, seekRect.width);

    const progress = seekX / seekRect.width;
    const time = progress * durationSeconds;

    const desiredPreviewLeft = seekX - previewWidth / 2;
    const nextPreviewLeft = getPreviewLeft({
      desiredLeft: desiredPreviewLeft,
      seekWidth: seekRect.width,
      seekViewportLeft: seekRect.left,
      seekViewportRight: seekRect.right,
      previewWidth,
      isFullscreen,
    });

    setCursorLeft(seekX);
    setPreviewLeft(nextPreviewLeft);
    setPreviewTime(time);
    setShowPreview(true);
  };

  const onSeek = () => {
    seek(previewTime);
    setShowPreview(false);
  };

  const progressPercent = durationSeconds ? (currentTime / durationSeconds) * 100 : 0;

  return (
    <div
      className="cursor-pointer pb-2"
      onPointerEnter={handlePointerMove}
      onPointerMove={handlePointerMove}
      onPointerLeave={() => setShowPreview(false)}
      onClick={onSeek}
    >
      <div className="mb-1 flex justify-between">
        <span className={cn("rounded-full px-1 py-0.5 text-xs font-medium", PLAYER_GLASS_CLASS)}>
          {formatTime(currentTime)}
        </span>

        <span className={cn("rounded-full px-1 py-0.5 text-xs font-medium", PLAYER_GLASS_CLASS)}>
          {formatTime(remainingSeconds)}
        </span>
      </div>

      <motion.div
        ref={seekRef}
        className={cn(PLAYER_GLASS_CLASS, "relative h-1 rounded-md bg-zinc-700/40")}
        animate={{ height: showPreview ? 6 : 4 }}
        transition={{ duration: 0.075 }}
      >
        {/* buffered ranges */}
        {durationSeconds > 0 &&
          bufferedRanges.map((range) => (
            <div
              key={range.start}
              className="absolute top-0 h-full rounded-md bg-white/25"
              style={{
                left: `${(range.start / durationSeconds) * 100}%`,
                width: `${((range.end - range.start) / durationSeconds) * 100}%`,
              }}
            />
          ))}

        {/* current progress */}
        <div className="absolute left-0 top-0 h-full rounded-md bg-white/80" style={{ width: `${progressPercent}%` }} />

        {/* cursor marker */}
        <div
          className="pointer-events-none absolute top-1/2 h-full w-0.5 -translate-y-1/2 rounded-full bg-zinc-300"
          style={{
            display: showPreview ? "block" : "none",
            left: cursorLeft,
            mixBlendMode: "difference",
          }}
        />

        {/* preview thumbnail/timestamp */}
        <div
          ref={previewRef}
          className="pointer-events-none absolute bottom-full mb-2"
          style={{
            visibility: showPreview ? "visible" : "hidden",
            left: previewLeft,
          }}
        >
          <div className="flex flex-col items-center gap-1">
            {previewFrame ? (
              <div
                className="overflow-hidden rounded bg-black shadow-lg"
                style={{
                  width: 256,
                  height: Math.max(1, (previewFrame.frameHeightPx / previewFrame.frameWidthPx) * 256),
                }}
              >
                <div
                  style={{
                    width: previewFrame.frameWidthPx,
                    height: previewFrame.frameHeightPx,
                    transform: `scale(${256 / previewFrame.frameWidthPx})`,
                    transformOrigin: "top left",
                    backgroundImage: `url(${previewFrame.assetSignedUrl})`,
                    backgroundPosition: `-${previewFrame.offsetXPx}px -${previewFrame.offsetYPx}px`,
                    backgroundSize: `${previewFrame.sheetWidthPx}px ${previewFrame.sheetHeightPx}px`,
                    backgroundRepeat: "no-repeat",
                  }}
                />
              </div>
            ) : null}
            <div className="rounded-md bg-zinc-800/80 px-1 py-0.5 text-xs font-medium backdrop-blur-2xl">
              {formatTime(previewTime)}
            </div>
          </div>
        </div>
      </motion.div>
    </div>
  );
};
