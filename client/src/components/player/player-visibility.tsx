import { createContext, useContext, useEffect, useMemo, useRef, useState, type FC, type PropsWithChildren } from "react";

interface PlayerVisibilityContextValue {
  showControls: boolean;
  showControlsTemporarily: () => void;
  hideControlsImmediately: () => void;
  lockControls: () => () => void;
}

const HIDE_DELAY_MS = 3000;
const PlayerVisibilityContext = createContext<PlayerVisibilityContextValue | null>(null);

export const PlayerVisibilityProvider: FC<PropsWithChildren> = ({ children }) => {
  const [showControls, setShowControls] = useState(true);
  const timeoutRef = useRef<number | null>(null);
  const locksRef = useRef(new Set<symbol>());

  const clearHideTimeout = () => {
    if (timeoutRef.current == null) return;
    window.clearTimeout(timeoutRef.current);
    timeoutRef.current = null;
  };

  const scheduleHide = () => {
    clearHideTimeout();
    if (locksRef.current.size > 0) return;
    timeoutRef.current = window.setTimeout(() => {
      if (locksRef.current.size === 0) {
        setShowControls(false);
      }
    }, HIDE_DELAY_MS);
  };

  const showControlsTemporarily = () => {
    setShowControls(true);
    scheduleHide();
  };

  const hideControlsImmediately = () => {
    clearHideTimeout();
    if (locksRef.current.size > 0) return;
    setShowControls(false);
  };

  const lockControls = () => {
    const token = Symbol("player-controls-lock");
    clearHideTimeout();
    locksRef.current.add(token);
    setShowControls(true);

    return () => {
      if (!locksRef.current.delete(token)) return;
      if (locksRef.current.size === 0) {
        scheduleHide();
      }
    };
  };

  useEffect(() => {
    return () => {
      clearHideTimeout();
      locksRef.current.clear();
    };
  }, []);

  const value = useMemo(
    () => ({
      showControls,
      showControlsTemporarily,
      hideControlsImmediately,
      lockControls,
    }),
    [showControls],
  );

  return <PlayerVisibilityContext.Provider value={value}>{children}</PlayerVisibilityContext.Provider>;
};

export const usePlayerVisibility = () => {
  const ctx = useContext(PlayerVisibilityContext);
  if (!ctx) {
    throw new Error("usePlayerVisibility must be used inside PlayerVisibilityProvider");
  }
  return ctx;
};

export const useShowControlsLock = (active: boolean) => {
  const { lockControls } = usePlayerVisibility();

  useEffect(() => {
    if (!active) return;
    return lockControls();
  }, [active, lockControls]);
};
