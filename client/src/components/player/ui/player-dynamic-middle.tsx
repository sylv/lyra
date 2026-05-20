import type { FC, ReactNode } from "react";
import { cn } from "../../../lib/utils";
import { useShowControls } from "../store/player-controls-store";
import { usePlayerStore } from "../store/player-store";

export const PlayerDynamicMiddle: FC<{ children: ReactNode }> = ({ children }) => {
  const showControls = useShowControls();
  const paused = usePlayerStore((state) => state.paused);
  const controlsVisible = showControls || paused;

  return (
    <div
      className={cn("pointer-events-none absolute inset-x-0 top-0 z-20", controlsVisible ? "bottom-28" : "bottom-5")}
    >
      {children}
    </div>
  );
};
