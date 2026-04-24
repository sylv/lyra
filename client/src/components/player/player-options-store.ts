import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import { useStore } from "zustand/react";

export interface PlayerSnapshot {
  currentItemId: string;
  position: number;
}

interface PlayerOptionsState {
  volume: number;
  isMuted: boolean;
  autoplayNext: boolean;
  snapshot: PlayerSnapshot | null;
  setVolume: (volume: number) => void;
  setMuted: (isMuted: boolean) => void;
  setAutoplayNext: (autoplayNext: boolean) => void;
  setSnapshot: (snapshot: PlayerSnapshot | null) => void;
}

const clampVolume = (volume: number) => Math.max(0, Math.min(1, volume));

export const playerOptionsStore = create<PlayerOptionsState>()(
  persist(
    (set) => ({
      volume: 1,
      isMuted: false,
      autoplayNext: true,
      snapshot: null,
      setVolume: (volume) => set({ volume: clampVolume(volume) }),
      setMuted: (isMuted) => set({ isMuted }),
      setAutoplayNext: (autoplayNext) => set({ autoplayNext }),
      setSnapshot: (snapshot) => set({ snapshot }),
    }),
    {
      name: "lyra.player.options",
      storage: createJSONStorage(() => window.localStorage),
      partialize: (state) => ({
        volume: state.volume,
        isMuted: state.isMuted,
        autoplayNext: state.autoplayNext,
        snapshot: state.snapshot,
      }),
    },
  ),
);

export const usePlayerOptionsStore = <T>(selector: (state: PlayerOptionsState) => T) =>
  useStore(playerOptionsStore, selector);
