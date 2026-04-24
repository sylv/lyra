import { AnimatePresence, motion } from "motion/react";
import { useState, type FC } from "react";
import { formatPlayerTime } from "../../../lib/format-player-time";
import { cn } from "../../../lib/utils";
import { usePlayerVisibility } from "../player-visibility";
import { PlayerNavigationFragment, PlayerLeftControls } from "./player-left-controls";
import { PlayerProgressBar, PlayerTimelinePreviewSheetFragment } from "./player-progress-bar";
import { PlayerRightControls } from "./player-right-controls";
import { PlayerItemCard } from "./player-item-card";
import type { FragmentType } from "../../../@generated/gql";

export const PlayerBottomBar: FC<{
  currentTime: number;
  duration: number;
  compact?: boolean;
  previousPlayable: FragmentType<typeof PlayerNavigationFragment> | null;
  nextPlayable: FragmentType<typeof PlayerNavigationFragment> | null;
  timelinePreviewSheets: FragmentType<typeof PlayerTimelinePreviewSheetFragment>[];
  portalContainer: HTMLElement | null;
}> = ({ currentTime, duration, compact = false, previousPlayable, nextPlayable, timelinePreviewSheets, portalContainer }) => {
  const { showControls } = usePlayerVisibility();
  const [hoveredPreview, setHoveredPreview] = useState<"previous" | "next" | null>(null);
  const previewItem = hoveredPreview === "previous" ? previousPlayable : hoveredPreview === "next" ? nextPlayable : null;

  return (
    <div
      data-player-interactive-root
      className={cn(
        "cursor-default transition-opacity duration-300",
        showControls ? "pointer-events-auto opacity-100" : "pointer-events-none opacity-0",
        compact ? "px-3 pb-3 pt-2" : "px-4 pb-4 pt-3",
      )}
    >
      <div className={cn("mb-1 flex justify-between text-white/80", compact ? "text-xs" : "text-sm")}>
        <span className="inline-flex items-center rounded-full bg-black/30 px-3 py-1">{formatPlayerTime(currentTime)}</span>
        <span className="inline-flex items-center rounded-full bg-black/30 px-3 py-1">{formatPlayerTime(duration)}</span>
      </div>

      <div className="relative">
        <PlayerProgressBar compact={compact} currentTime={currentTime} duration={duration} timelinePreviewSheets={timelinePreviewSheets} />
        {!compact ? (
          <div className="pointer-events-none absolute bottom-8 left-0">
            <AnimatePresence mode="wait">
              {previewItem ? (
                <motion.div
                  key={hoveredPreview}
                  initial={{ opacity: 0, translateX: -12 }}
                  animate={{ opacity: 1, translateX: 0 }}
                  exit={{ opacity: 0, translateX: -12 }}
                  transition={{ duration: 0.1 }}
                  className="pointer-events-auto"
                >
                  <PlayerItemCard item={previewItem} />
                </motion.div>
              ) : null}
            </AnimatePresence>
          </div>
        ) : null}
      </div>

      <div className={cn("flex items-center justify-between", compact ? "mt-1" : "mt-1.5")}>
        <PlayerLeftControls previousPlayable={previousPlayable} nextPlayable={nextPlayable} onHoverPreviewChange={setHoveredPreview} />
        <PlayerRightControls currentTime={currentTime} duration={duration} portalContainer={portalContainer} />
      </div>
    </div>
  );
};
