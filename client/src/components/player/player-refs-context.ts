import { createContext, type RefObject, useContext } from "react";
import type { PlayerController } from "./hls";

interface PlayerRefsContextValue {
  videoRef: RefObject<HTMLVideoElement | null>;
  controllerRef: RefObject<PlayerController | null>;
  containerRef: RefObject<HTMLDivElement | null>;
  surfaceRef: RefObject<HTMLDivElement | null>;
}

export const PlayerRefsContext = createContext<PlayerRefsContextValue | null>(null);

export const usePlayerRefsContext = (): PlayerRefsContextValue => {
  const ctx = useContext(PlayerRefsContext);
  if (!ctx) {
    throw new Error("usePlayerRefsContext must be used inside PlayerRefsContext.Provider");
  }
  return ctx;
};
