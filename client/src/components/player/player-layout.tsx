import type { FC, ReactNode } from "react";

interface PlayerLayoutProps {
  top: ReactNode;
  middle: ReactNode;
  bottom: ReactNode;
}

// three-zone overlay layout using absolute positioning, matching the original top/bottom chrome behavior.
// top and bottom zones overlay the video; middle covers the full area for floating content (skip intro, up next, etc.).
// all zones are pointer-events-none by default; interactive children opt in with pointer-events-auto.
export const PlayerLayout: FC<PlayerLayoutProps> = ({ top, middle, bottom }) => {
  return (
    <div className="absolute inset-0 pointer-events-none">
      <div className="absolute top-0 left-0 right-0 z-30">{top}</div>
      <div className="absolute inset-0 z-20">{middle}</div>
      <div className="absolute bottom-0 left-0 right-0 z-30">{bottom}</div>
    </div>
  );
};
