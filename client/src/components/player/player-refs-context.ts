import { createContext, type RefObject, useContext } from "react";
import type { PlayerController } from "./hls";

interface PlayerRefsContextValue {
  controllerRef: RefObject<PlayerController | null>;
  containerRef: RefObject<HTMLDivElement | null>;
  surfaceRef: RefObject<HTMLDivElement | null>;
}

interface PlayerVideoElementContextValue {
  videoElement: HTMLVideoElement | null;
  setVideoElement: (element: HTMLVideoElement | null) => void;
}

export const PlayerRefsContext = createContext<PlayerRefsContextValue | null>(null);
export const PlayerVideoElementContext = createContext<PlayerVideoElementContextValue | undefined>(undefined);

export const usePlayerRefsContext = (): PlayerRefsContextValue => {
  const ctx = useContext(PlayerRefsContext);
  if (!ctx) {
    throw new Error("usePlayerRefsContext must be used inside PlayerRefsContext.Provider");
  }
  return ctx;
};

export const usePlayerVideoElement = (): HTMLVideoElement | null => {
  const ctx = useContext(PlayerVideoElementContext);
  if (ctx === undefined) {
    throw new Error("usePlayerVideoElement must be used inside PlayerVideoElementContext.Provider");
  }
  return ctx.videoElement;
};

export const usePlayerVideoElementRegistration = (): ((element: HTMLVideoElement | null) => void) => {
  const ctx = useContext(PlayerVideoElementContext);
  if (ctx === undefined) {
    throw new Error("usePlayerVideoElementRegistration must be used inside PlayerVideoElementContext.Provider");
  }
  return ctx.setVideoElement;
};
