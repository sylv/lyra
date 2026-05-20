import React, { createContext } from "react";

interface PlayerVideoContext {
  togglePlaying: () => void;
  toggleSurfacePlaying: () => void;
  seek: (time: number) => void;
  toggleMute: () => void;
}

export const PlayerVideoContext = createContext<PlayerVideoContext | null>(null);

export const useVideoControls = () => {
  const context = React.useContext(PlayerVideoContext);
  if (!context) throw new Error("useVideoControls must be used within a PlayerVideoProvider");
  return context;
};
