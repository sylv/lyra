import { motion } from "motion/react";
import { cn } from "../../../lib/utils";

interface SkipIntroButtonProps {
  progressPercent: number;
  onSkip: () => void;
}

export const SkipIntroButton = ({ progressPercent, onSkip }: SkipIntroButtonProps) => {
  const clampedProgressPercent = Math.max(0, Math.min(100, progressPercent * 100));

  return (
    <motion.button
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 8 }}
      transition={{ duration: 0.2 }}
      type="button"
      onClick={(event) => {
        // or else the click will pause the player by propogating up and being considered
        // a click on the video.
        event.stopPropagation();
        onSkip();
      }}
      className={cn(
        "relative overflow-hidden rounded-md bg-white/70 px-3 py-2 text-left text-black shadow-lg backdrop-blur-sm transition-colors hover:bg-white/50",
      )}
    >
      <div className="pointer-events-none absolute inset-0">
        <div
          className="h-full bg-white/90 transition-[width] duration-300 ease-linear"
          style={{
            width: `${clampedProgressPercent}%`,
          }}
        />
      </div>

      <div className="relative z-10 flex items-center gap-3 px-6">
        <span className="text-sm font-semibold">Skip Intro</span>
      </div>
    </motion.button>
  );
};
