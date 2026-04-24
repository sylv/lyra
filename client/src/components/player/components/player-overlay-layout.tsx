import type { FC, ReactNode } from "react";

interface PlayerOverlayLayoutProps {
  top: ReactNode;
  middle: ReactNode;
  bottom: ReactNode;
}

// This keeps the player shell responsible for placement while overlays own behavior.
export const PlayerOverlayLayout: FC<PlayerOverlayLayoutProps> = ({ top, middle, bottom }) => {
  return (
    <div className="absolute inset-0 pointer-events-none">
      <div className="absolute left-0 right-0 top-0 z-30">{top}</div>
      <div className="absolute inset-0 z-20">{middle}</div>
      <div className="absolute bottom-0 left-0 right-0 z-30">{bottom}</div>
    </div>
  );
};
