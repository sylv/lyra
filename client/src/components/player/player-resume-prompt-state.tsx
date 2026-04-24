import { createContext, useContext, useMemo, useState, type FC, type PropsWithChildren } from "react";

interface PlayerResumePromptContextValue {
  positionSeconds: number | null;
  openPrompt: (positionSeconds: number, handlers: { resume: () => void; startOver: () => void }) => void;
  confirmPrompt: () => void;
  cancelPrompt: () => void;
}

const PlayerResumePromptContext = createContext<PlayerResumePromptContextValue | null>(null);

export const PlayerResumePromptProvider: FC<PropsWithChildren> = ({ children }) => {
  const [positionSeconds, setPositionSeconds] = useState<number | null>(null);
  const [handlers, setHandlers] = useState<{ resume: () => void; startOver: () => void } | null>(null);

  const value = useMemo<PlayerResumePromptContextValue>(
    () => ({
      positionSeconds,
      openPrompt: (nextPositionSeconds, nextHandlers) => {
        setPositionSeconds(nextPositionSeconds);
        setHandlers(nextHandlers);
      },
      confirmPrompt: () => {
        handlers?.resume();
        setPositionSeconds(null);
        setHandlers(null);
      },
      cancelPrompt: () => {
        handlers?.startOver();
        setPositionSeconds(null);
        setHandlers(null);
      },
    }),
    [handlers, positionSeconds],
  );

  return <PlayerResumePromptContext.Provider value={value}>{children}</PlayerResumePromptContext.Provider>;
};

export const usePlayerResumePrompt = () => {
  const ctx = useContext(PlayerResumePromptContext);
  if (!ctx) {
    throw new Error("usePlayerResumePrompt must be used inside PlayerResumePromptProvider");
  }
  return ctx;
};
