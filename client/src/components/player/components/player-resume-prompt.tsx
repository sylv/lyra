import type { FC } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../../ui/dialog";
import { usePlayerResumePrompt } from "../player-resume-prompt-state";
import { useShowControlsLock } from "../player-visibility";

const formatResumeTimestamp = (seconds: number): string => {
  const safeSeconds = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(safeSeconds / 3600);
  const minutes = Math.floor((safeSeconds % 3600) / 60);
  const remainingSeconds = safeSeconds % 60;
  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, "0")}:${remainingSeconds.toString().padStart(2, "0")}`;
  }
  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
};

export const PlayerResumePrompt: FC<{ portalContainer: HTMLElement | null }> = ({ portalContainer }) => {
  const { positionSeconds, confirmPrompt, cancelPrompt } = usePlayerResumePrompt();
  useShowControlsLock(positionSeconds != null);

  return (
    <Dialog
      open={positionSeconds != null}
      onOpenChange={(open) => {
        if (!open) cancelPrompt();
      }}
    >
      <DialogContent
        portalContainer={portalContainer}
        className="max-w-sm gap-0 overflow-hidden p-0 [&>button.absolute]:hidden"
        onClick={(event) => event.stopPropagation()}
      >
        <DialogHeader className="sr-only">
          <DialogTitle>Choose playback start</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col">
          <button
            type="button"
            className="w-full border-b border-zinc-700/80 bg-zinc-900/95 px-5 py-4 text-left text-sm font-semibold transition-colors hover:bg-zinc-800/95"
            onClick={confirmPrompt}
          >
            Resume from {positionSeconds == null ? "0:00" : formatResumeTimestamp(positionSeconds)}
          </button>
          <button
            type="button"
            className="w-full bg-zinc-900/95 px-5 py-4 text-left text-sm font-semibold transition-colors hover:bg-zinc-800/95"
            onClick={cancelPrompt}
          >
            Start from the beginning
          </button>
        </div>
      </DialogContent>
    </Dialog>
  );
};
