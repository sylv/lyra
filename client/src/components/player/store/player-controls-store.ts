import { useEffect, useId } from "react";
import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

interface PlayerControlsStore {
  mouseLastMoved: number;
  mouseIsHovering: boolean;
  forceControlIds: string[];
  tick: number;
}

export const usePlayerControlsStore = create<PlayerControlsStore>()(
  immer(() => ({
    mouseLastMoved: Date.now(),
    mouseIsHovering: false,
    forceControlIds: [] as string[],
    tick: 0,
  })),
);

export const bumpPlayerControls = () => {
  usePlayerControlsStore.setState((state) => {
    state.mouseLastMoved = Date.now();
    state.tick += 1;
  });
};

export const useControlsOverride = (show: boolean) => {
  const id = useId();
  useEffect(() => {
    if (show) {
      usePlayerControlsStore.setState((state) => {
        if (!state.forceControlIds.includes(id)) {
          state.forceControlIds.push(id);
        }
      });
    } else {
      usePlayerControlsStore.setState((state) => {
        state.forceControlIds = state.forceControlIds.filter((forceId) => forceId !== id);
      });
    }

    return () => {
      usePlayerControlsStore.setState((state) => {
        state.forceControlIds = state.forceControlIds.filter((forceId) => forceId !== id);
      });
    };
  }, [id, show]);
};

export const useShowControls = () => {
  const mouseLastMoved = usePlayerControlsStore((state) => state.mouseLastMoved);
  const mouseIsHovering = usePlayerControlsStore((state) => state.mouseIsHovering);
  const forceControlIds = usePlayerControlsStore((state) => state.forceControlIds);
  const tick = usePlayerControlsStore((state) => state.tick);

  useEffect(() => {
    const interval = window.setInterval(() => {
      usePlayerControlsStore.setState((state) => {
        state.tick += 1;
      });
    }, 250);
    return () => window.clearInterval(interval);
  }, []);

  void tick;
  if (forceControlIds.length > 0) return true;
  if (mouseIsHovering && Date.now() - mouseLastMoved < 3000) return true;
  return false;
};
