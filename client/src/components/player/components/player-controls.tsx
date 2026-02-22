/** biome-ignore-all lint/a11y/useKeyWithClickEvents: <explanation> */
/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
import { MaximizeIcon, MinimizeIcon, PauseIcon, PlayIcon, SettingsIcon } from "lucide-react";
import { useMemo, type FC } from "react";
import { cn } from "../../../lib/utils";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuRadioGroup,
	DropdownMenuRadioItem,
	DropdownMenuSeparator,
	DropdownMenuSub,
	DropdownMenuSubContent,
	DropdownMenuSubTrigger,
	DropdownMenuTrigger,
} from "../../ui/dropdown-menu";
import { formatPlayerTime } from "../utils";
import { PaddedPlayerButton, PlayerButton } from "./player-button";
import { PlayerProgressBar } from "./player-progress-bar";
import { PlayerVolumeControl } from "./player-volume-control";

interface PlayerAudioTrackOption {
	id: number;
	label: string;
}

interface PlayerTimelinePreviewSheet {
	positionMs: number;
	endMs: number;
	sheetIntervalMs: number;
	sheetGapSize: number;
	asset: {
		id: number;
		width?: number | null;
		height?: number | null;
	};
}

interface PlayerControlsProps {
	showControls: boolean;
	isFullscreen: boolean;
	currentTime: number;
	duration: number;
	bufferedRanges: { start: number; end: number }[];
	timelinePreviewSheets: PlayerTimelinePreviewSheet[];
	playing: boolean;
	volume: number;
	isMuted: boolean;
	onSeek: (time: number) => void;
	onTogglePlaying: () => void;
	onToggleMute: () => void;
	onVolumeChange: (volume: number) => void;
	onToggleFullscreen: () => void;
	audioTrackOptions: PlayerAudioTrackOption[];
	selectedAudioTrackId: number | null;
	onAudioTrackChange: (trackId: number) => void;
	onOpenShortcuts: () => void;
	isSettingsMenuOpen: boolean;
	onSettingsMenuOpenChange: (open: boolean) => void;
	dropdownPortalContainer: HTMLElement | null;
}

export const PlayerControls: FC<PlayerControlsProps> = ({
	showControls,
	isFullscreen,
	currentTime,
	duration,
	bufferedRanges,
	timelinePreviewSheets,
	playing,
	volume,
	isMuted,
	onSeek,
	onTogglePlaying,
	onToggleMute,
	onVolumeChange,
	onToggleFullscreen,
	audioTrackOptions,
	selectedAudioTrackId,
	onAudioTrackChange,
	onOpenShortcuts,
	isSettingsMenuOpen,
	onSettingsMenuOpenChange,
	dropdownPortalContainer,
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
				timelinePreviewSheets={timelinePreviewSheets}
				onChange={onSeek}
			/>

			{/* Control buttons */}
			<div className="flex items-center justify-between">
				{/* Left side */}
				<div className="flex items-center gap-2">
					<PaddedPlayerButton onClick={onTogglePlaying} side="left">
						{playing ? <PauseIcon className="size-6 text-white" /> : <PlayIcon className="size-6 text-white" />}
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
					<DropdownMenu open={isSettingsMenuOpen} onOpenChange={onSettingsMenuOpenChange}>
						<DropdownMenuTrigger asChild>
							<PlayerButton
								aria-label="Open player settings"
								onClick={(event) => {
									event.stopPropagation();
								}}
							>
								<SettingsIcon className="size-5" />
							</PlayerButton>
						</DropdownMenuTrigger>
						<DropdownMenuContent
							align="end"
							portalContainer={dropdownPortalContainer}
							onClick={(event) => event.stopPropagation()}
							className="z-[70] w-56 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40"
						>
							<DropdownMenuSub>
								<DropdownMenuSubTrigger className="py-2.5 data-[state=open]:bg-zinc-800 focus:bg-zinc-800">
									Audio
								</DropdownMenuSubTrigger>
								<DropdownMenuSubContent className="z-[70] border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40">
									{audioTrackOptions.length === 0 ? (
										<DropdownMenuItem className="py-2.5" disabled>
											No audio tracks
										</DropdownMenuItem>
									) : (
										<DropdownMenuRadioGroup
											value={selectedAudioTrackId?.toString()}
											onValueChange={(value) => onAudioTrackChange(Number.parseInt(value, 10))}
										>
											{audioTrackOptions.map((track) => (
												<DropdownMenuRadioItem
													className="py-2.5 focus:bg-zinc-800"
													key={track.id}
													value={track.id.toString()}
												>
													{track.label}
												</DropdownMenuRadioItem>
											))}
										</DropdownMenuRadioGroup>
									)}
								</DropdownMenuSubContent>
							</DropdownMenuSub>
							<DropdownMenuSeparator className="bg-zinc-700/80" />
							<DropdownMenuItem className="py-2.5 focus:bg-zinc-800" onSelect={onOpenShortcuts}>
								Shortcuts
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
					<PaddedPlayerButton
						side="right"
						aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
						onClick={(e) => {
							e.stopPropagation();
							onToggleFullscreen();
						}}
					>
						{isFullscreen ? <MinimizeIcon className="size-5" /> : <MaximizeIcon className="size-5" />}
					</PaddedPlayerButton>
				</div>
			</div>
		</div>
	);
};
