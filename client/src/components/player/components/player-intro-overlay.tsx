import { useMemo, type FC } from "react";
import { useStore } from "zustand/react";
import { AnimatePresence } from "motion/react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { playerState } from "../player-state";
import { usePlayerActions } from "../hooks/use-player-actions";
import { videoState } from "../video-state";
import { SkipIntroButton } from "./skip-intro-button";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

interface PlayerIntroOverlayProps {
	media: CurrentMedia;
}

export const PlayerIntroOverlay: FC<PlayerIntroOverlayProps> = ({ media }) => {
	const currentTime = useStore(videoState, (s) => s.currentTime);
	const isFullscreen = useStore(playerState, (s) => s.isFullscreen);
	const { onSeek } = usePlayerActions();

	const introSegment = useMemo(() => {
		const segments = media.file?.segments;
		if (!Array.isArray(segments)) return null;
		return (
			segments.find(
				(segment) =>
					segment.kind === "INTRO" &&
					typeof segment.startMs === "number" &&
					typeof segment.endMs === "number" &&
					segment.endMs > segment.startMs,
			) ?? null
		);
	}, [media.file?.segments]);

	const introProgressPercent = useMemo(() => {
		if (!introSegment) return 0;
		const introDurationMs = introSegment.endMs - introSegment.startMs;
		if (introDurationMs <= 0) return 0;
		const positionMs = currentTime * 1000;
		return Math.max(0, Math.min(1, (positionMs - introSegment.startMs) / introDurationMs));
	}, [currentTime, introSegment]);

	const isInsideIntroSegment = useMemo(() => {
		if (!introSegment) return false;
		const positionMs = currentTime * 1000;
		return positionMs >= introSegment.startMs && positionMs < introSegment.endMs;
	}, [currentTime, introSegment]);

	const showButton = !!introSegment && isInsideIntroSegment && !!isFullscreen;

	return (
		<div className="absolute right-0 flex justify-end px-4 pointer-events-none bottom-36">
			<div className="pointer-events-auto">
				<AnimatePresence>
					{showButton && introSegment && (
						<SkipIntroButton
							key="skip-intro"
							progressPercent={introProgressPercent}
							onSkip={() => onSeek(introSegment.endMs / 1000)}
						/>
					)}
				</AnimatePresence>
			</div>
		</div>
	);
};
