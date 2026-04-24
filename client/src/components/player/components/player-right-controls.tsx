import { MaximizeIcon, MinimizeIcon } from "lucide-react";
import { useMemo, type FC } from "react";
import { togglePlayerFullscreen, usePlayerRuntimeStore } from "../player-runtime-store";
import { PlayerButton } from "../ui/player-button";
import { PlayerSettings } from "./player-settings";

export const PlayerRightControls: FC<{
  currentTime: number;
  duration: number;
  portalContainer: HTMLElement | null;
}> = ({ currentTime, duration, portalContainer }) => {
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);

  const finishTime = useMemo(() => {
    if (!duration || !currentTime) return null;
    const remainingTimeMs = (duration - currentTime) * 1000;
    const finishDate = new Date(Date.now() + remainingTimeMs);
    return finishDate.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }, [currentTime, duration]);

  return (
    <div className="flex items-center gap-1 rounded-full bg-black/30 p-1">
      {finishTime && isFullscreen ? <span className="px-3 text-sm">Finishes at {finishTime}</span> : null}
      <PlayerSettings portalContainer={portalContainer} />
      <PlayerButton
        aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
        onClick={(event) => {
          event.stopPropagation();
          togglePlayerFullscreen();
        }}
      >
        {isFullscreen ? <MinimizeIcon className="size-5" /> : <MaximizeIcon className="size-5" />}
      </PlayerButton>
    </div>
  );
};
