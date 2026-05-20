import { Maximize2Icon, Minimize2Icon, PauseIcon, PlayIcon, SkipBackIcon, SkipForwardIcon } from "lucide-react";
import { useMemo, useState, type FC } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "../../ui/tooltip";
import { PlayerSeekBar } from "./player-seek-bar";
import { cn } from "../../../lib/utils";
import { PLAYER_GLASS_CLASS } from "../constants";
import { PlayerState, playNode, usePlayerStore } from "../store/player-store";
import { useVideoControls } from "../store/player-video-context";
import { PlayerItemCard } from "../components/player-item-card";
import { PlayerVolumeControl } from "../components/player-volume-control";
import { PlayerSettings } from "../components/player-settings";
import { useControlsOverride } from "../store/player-controls-store";

export const PlayerBottom: FC<{ portalContainer: HTMLElement | null }> = ({ portalContainer }) => {
  const { togglePlaying } = useVideoControls();
  const playing = usePlayerStore((state) => !state.paused);
  const isFullscreen = usePlayerStore((state) => state.isFullscreen);
  const duration = usePlayerStore((state) => state.durationSeconds);
  const currentTime = usePlayerStore((state) => state.currentTime);
  const status = usePlayerStore((state) => state.status);
  const [previewItem, setPreviewItem] = useState<"previous" | "next" | null>(null);
  useControlsOverride(previewItem != null);
  const finishTime = useMemo(() => {
    if (!duration || !currentTime) return null;
    const remainingTimeMs = (duration - currentTime) * 1000;
    const finishDate = new Date(Date.now() + remainingTimeMs);
    return finishDate.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }, [currentTime, duration]);
  const previousPlayable = status.state === PlayerState.Mounted ? status.data.node.previousPlayable : null;
  const nextPlayable = status.state === PlayerState.Mounted ? status.data.node.nextPlayable : null;
  const previewNode = previewItem === "previous" ? previousPlayable : previewItem === "next" ? nextPlayable : null;
  const buttonClassName = "hover:bg-zinc-600/30 p-1.5 rounded-full relative";

  return (
    <div className="relative p-3">
      {previewNode ? (
        <div className="pointer-events-none absolute bottom-full left-3 mb-3">
          <PlayerItemCard item={previewNode} />
        </div>
      ) : null}
      <div className="flex flex-col gap-3">
        <div className="-mb-3">
          <PlayerSeekBar />
        </div>
        <div className="flex items-center justify-between">
          {/* left side */}
          <div className={cn("flex items-center gap-1 p-1.5 rounded-full", PLAYER_GLASS_CLASS)}>
            <PlayerButton
              icon={playing ? <PauseIcon className="size-6" /> : <PlayIcon className="size-6" />}
              name={playing ? "Pause" : "Play"}
              shortcut="Space"
              onClick={() => {
                togglePlaying();
              }}
            />
            <PlayerButton
              icon={<SkipBackIcon className="size-6" />}
              name="Previous"
              disabled={!previousPlayable}
              onPointerEnter={() => setPreviewItem("previous")}
              onPointerLeave={() => setPreviewItem(null)}
              onClick={() => previousPlayable && playNode(previousPlayable.id, isFullscreen)}
            />
            <PlayerButton
              icon={<SkipForwardIcon className="size-6" />}
              name="Next"
              disabled={!nextPlayable}
              onPointerEnter={() => setPreviewItem("next")}
              onPointerLeave={() => setPreviewItem(null)}
              onClick={() => nextPlayable && playNode(nextPlayable.id, isFullscreen)}
            />
            <PlayerVolumeControl buttonClassName={buttonClassName} />
          </div>
          {/* right side */}
          <div className={cn("flex items-center gap-1 p-1.5 rounded-full", PLAYER_GLASS_CLASS)}>
            {finishTime && isFullscreen ? <span className="px-3 text-sm">Finishes at {finishTime}</span> : null}
            <PlayerSettings buttonClassName={buttonClassName} portalContainer={portalContainer} />
            <PlayerButton
              icon={isFullscreen ? <Minimize2Icon className="size-6" /> : <Maximize2Icon className="size-6" />}
              name="Fullscreen"
              shortcut="F"
              onClick={() => {
                usePlayerStore.setState((state) => {
                  state.isFullscreen = !state.isFullscreen;
                });
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

const PlayerButton: FC<{
  icon: React.ReactNode;
  name: string;
  shortcut?: string;
  onClick?: () => void;
  disabled?: boolean;
  onPointerEnter?: () => void;
  onPointerLeave?: () => void;
}> = ({ icon, name, shortcut, onClick, disabled, onPointerEnter, onPointerLeave }) => {
  return (
    <Tooltip>
      <TooltipContent>
        {name} {shortcut && <kbd>{shortcut}</kbd>}
      </TooltipContent>
      <TooltipTrigger asChild>
        <button
          className="hover:bg-zinc-600/30 disabled:opacity-40 disabled:hover:bg-transparent p-1.5 rounded-full"
          disabled={disabled}
          onClick={onClick}
          onPointerEnter={onPointerEnter}
          onPointerLeave={onPointerLeave}
        >
          {icon}
        </button>
      </TooltipTrigger>
    </Tooltip>
  );
};
