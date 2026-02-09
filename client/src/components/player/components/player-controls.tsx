/** biome-ignore-all lint/a11y/useKeyWithClickEvents: <explanation> */
/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
import { MaximizeIcon, MinimizeIcon, PauseIcon, PlayIcon } from "lucide-react";
import { useMemo, type FC } from "react";
import { cn } from "../../../lib/utils";
import { formatPlayerTime } from "../utils";
import { PaddedPlayerButton } from "./player-button";
import { PlayerProgressBar } from "./player-progress-bar";
import { PlayerVolumeControl } from "./player-volume-control";

interface PlayerControlsProps {
	showControls: boolean;
	isFullscreen: boolean;
	currentTime: number;
	duration: number;
	bufferedRanges: { start: number; end: number }[];
	playing: boolean;
	volume: number;
	isMuted: boolean;
	onSeek: (time: number) => void;
	onTogglePlaying: () => void;
	onToggleMute: () => void;
	onVolumeChange: (volume: number) => void;
	onToggleFullscreen: () => void;
}

export const PlayerControls: FC<PlayerControlsProps> = ({
	showControls,
	isFullscreen,
	currentTime,
	duration,
	bufferedRanges,
	playing,
	volume,
	isMuted,
	onSeek,
	onTogglePlaying,
	onToggleMute,
	onVolumeChange,
	onToggleFullscreen,
}) => {
	// eg, "6:33pm"
	const finishTime = useMemo(() => {
		if (!duration || !currentTime) return null;
		const remainingTimeMs = (duration - currentTime) * 1000;
		const finishDate = new Date(Date.now() + remainingTimeMs);
		return finishDate.toLocaleTimeString([], {
			hour: "2-digit",
			minute: "2-digit",
		});
	}, [duration, currentTime]);

	return (
		<div
			onClick={(event) => event.stopPropagation()}
			className={cn(
				"absolute bottom-0 left-0 right-0 transition-opacity duration-300 group cursor-default !pt-1",
				showControls ? "opacity-100" : "opacity-0",
				isFullscreen ? "p-6" : "p-4",
			)}
		>
			{/* Time indicators */}
			<div className="flex justify-between text-sm text-white/80">
				<span>{formatPlayerTime(currentTime)}</span>
				<span>{formatPlayerTime(duration)}</span>
			</div>

			{/* Progress bar */}
			<PlayerProgressBar
				duration={duration}
				currentTime={currentTime}
				bufferedRanges={bufferedRanges}
				onChange={onSeek}
			/>

			{/* Control buttons */}
			<div className="flex items-center justify-between">
				{/* Left side */}
				<div className="flex items-center gap-2">
					<PaddedPlayerButton onClick={onTogglePlaying} side="left">
						{playing ? <PauseIcon className="w-6 h-6 text-white" /> : <PlayIcon className="w-6 h-6 text-white" />}
					</PaddedPlayerButton>
					<PlayerVolumeControl
						volume={volume}
						isMuted={isMuted}
						onVolumeChange={onVolumeChange}
						onToggleMute={onToggleMute}
					/>
				</div>
				{/* Right side */}
				<div className="flex items-center gap-4">
					{finishTime && <span className="text-sm">Finishes at {finishTime}</span>}
					<PaddedPlayerButton
						side="right"
						aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
						onClick={(e) => {
							e.stopPropagation();
							onToggleFullscreen();
						}}
					>
						{isFullscreen ? <MinimizeIcon className="w-5 h-5" /> : <MaximizeIcon className="w-5 h-5" />}
					</PaddedPlayerButton>
				</div>
			</div>
		</div>
	);
};
