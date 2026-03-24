import { createContext, type RefObject, useContext } from "react";
import type { PlaybackEngine } from "./engines";

interface PlayerContextValue {
	videoRef: RefObject<HTMLVideoElement | null>;
	engineRef: RefObject<PlaybackEngine | null>;
	containerRef: RefObject<HTMLDivElement | null>;
	surfaceRef: RefObject<HTMLDivElement | null>;
}

export const PlayerContext = createContext<PlayerContextValue | null>(null);

export const usePlayerContext = (): PlayerContextValue => {
	const ctx = useContext(PlayerContext);
	if (!ctx) {
		throw new Error("usePlayerContext must be used inside PlayerContext.Provider");
	}
	return ctx;
};
