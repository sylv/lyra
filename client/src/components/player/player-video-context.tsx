import { createContext, useContext, useMemo, useState, type FC, type PropsWithChildren } from "react";

interface PlayerVideoContextValue {
  videoElement: HTMLVideoElement | null;
  setVideoElement: (element: HTMLVideoElement | null) => void;
}

const PlayerVideoContext = createContext<PlayerVideoContextValue | null>(null);

export const PlayerVideoProvider: FC<PropsWithChildren> = ({ children }) => {
  const [videoElement, setVideoElement] = useState<HTMLVideoElement | null>(null);
  const value = useMemo(
    () => ({
      videoElement,
      setVideoElement,
    }),
    [videoElement],
  );

  return <PlayerVideoContext.Provider value={value}>{children}</PlayerVideoContext.Provider>;
};

export const usePlayerVideoElement = () => {
  const ctx = useContext(PlayerVideoContext);
  if (!ctx) {
    throw new Error("usePlayerVideoElement must be used inside PlayerVideoProvider");
  }
  return ctx.videoElement;
};

export const usePlayerVideoRegistration = () => {
  const ctx = useContext(PlayerVideoContext);
  if (!ctx) {
    throw new Error("usePlayerVideoRegistration must be used inside PlayerVideoProvider");
  }
  return ctx.setVideoElement;
};
