/* oxlint-disable jsx_a11y/click-events-have-key-events, jsx_a11y/no-static-element-interactions */
import {
	MaximizeIcon,
	MinimizeIcon,
	PauseIcon,
	PlayIcon,
	SettingsIcon,
	SkipBackIcon,
	SkipForwardIcon,
} from "lucide-react";
import { useMemo, type FC } from "react";
import { useStore } from "zustand/react";
import type { FragmentType } from "../../../@generated/gql";
import { formatPlayerTime } from "../../../lib/format-player-time";
import { cn } from "../../../lib/utils";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuRadioGroup,
	DropdownMenuRadioItem,
	DropdownMenuSub,
	DropdownMenuSubContent,
	DropdownMenuSubTrigger,
	DropdownMenuTrigger,
} from "../../ui/dropdown-menu";
import { usePlayerActions } from "../hooks/use-player-actions";
import { playerState, togglePlayerFullscreen } from "../player-state";
import { videoState } from "../video-state";
import { PlayerButton } from "./player-button";
import { PlayerProgressBar, PlayerTimelinePreviewSheetFragment } from "./player-progress-bar";
import { PlayerVolumeControl } from "./player-volume-control";

interface PlayerControlsProps {
	timelinePreviewSheets: FragmentType<typeof PlayerTimelinePreviewSheetFragment>[];
	hasPreviousItem: boolean;
	hasNextItem: boolean;
	onPreviousItem: () => void;
	onNextItem: () => void;
	onAudioTrackChange: (trackId: number | null) => void;
	onSubtitleTrackChange: (trackId: number | null) => void;
	onControlsInteractionStart: () => void;
	onControlsInteractionEnd: () => void;
	onControlsActivity: () => void;
	dropdownPortalContainer: HTMLElement | null;
}

export const PlayerControls: FC<PlayerControlsProps> = ({
	timelinePreviewSheets,
	hasPreviousItem,
	hasNextItem,
	onPreviousItem,
	onNextItem,
	onAudioTrackChange,
	onSubtitleTrackChange,
	onControlsInteractionStart,
	onControlsInteractionEnd,
	onControlsActivity,
	dropdownPortalContainer,
}) => {
	const currentTime = useStore(videoState, (s) => s.currentTime);
	const duration = useStore(videoState, (s) => s.duration);
	const bufferedRanges = useStore(videoState, (s) => s.bufferedRanges);
	const playing = useStore(videoState, (s) => s.playing);
	const audioTrackOptions = useStore(videoState, (s) => s.audioTrackOptions);
	const selectedAudioTrackId = useStore(videoState, (s) => s.selectedAudioTrackId);
	const subtitleTrackOptions = useStore(videoState, (s) => s.subtitleTrackOptions);
	const selectedSubtitleTrackId = useStore(videoState, (s) => s.selectedSubtitleTrackId);
	const showControls = useStore(videoState, (s) => s.showControls);
	const isSettingsMenuOpen = useStore(videoState, (s) => s.isSettingsMenuOpen);
	const { volume, isMuted, isFullscreen } = useStore(playerState);
	const { onSeek, togglePlaying, onToggleMute, onVolumeChange } = usePlayerActions();

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
	const showItemNavigation = hasPreviousItem || hasNextItem;

	return (
		<div
			onClick={(event) => event.stopPropagation()}
			className={cn(
				"transition-opacity duration-300 group cursor-default !pt-1 pointer-events-auto",
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
				onInteractionStart={onControlsInteractionStart}
				onInteractionEnd={onControlsInteractionEnd}
				onActivity={onControlsActivity}
			/>

			{/* Control buttons */}
			<div className="flex items-center justify-between">
				{/* Left side */}
				<div className="flex items-center gap-2">
					<PlayerButton aria-label={playing ? "Pause" : "Play"} onClick={togglePlaying}>
						{playing ? <PauseIcon className="size-6 text-white" /> : <PlayIcon className="size-6 text-white" />}
					</PlayerButton>
					{showItemNavigation && (
						<>
							<PlayerButton
								aria-label="Previous item"
								disabled={!hasPreviousItem}
								onClick={(event) => {
									event.stopPropagation();
									if (hasPreviousItem) {
										onPreviousItem();
									}
								}}
							>
								<SkipBackIcon className="size-5" />
							</PlayerButton>
							<PlayerButton
								aria-label="Next item"
								disabled={!hasNextItem}
								onClick={(event) => {
									event.stopPropagation();
									if (hasNextItem) {
										onNextItem();
									}
								}}
							>
								<SkipForwardIcon className="size-5" />
							</PlayerButton>
						</>
					)}
					<PlayerVolumeControl
						volume={volume}
						isMuted={isMuted}
						onVolumeChange={onVolumeChange}
						onToggleMute={onToggleMute}
						onInteractionStart={onControlsInteractionStart}
						onInteractionEnd={onControlsInteractionEnd}
						onActivity={onControlsActivity}
					/>
				</div>
				{/* Right side */}
				<div className="flex items-center gap-4">
					{finishTime && <span className="text-sm">Finishes at {finishTime}</span>}
					<DropdownMenu
						open={isSettingsMenuOpen}
						onOpenChange={(open) => videoState.setState({ isSettingsMenuOpen: open })}
					>
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
											value={selectedAudioTrackId?.toString() ?? "auto"}
											onValueChange={(value) =>
												value === "auto"
													? onAudioTrackChange(null)
													: onAudioTrackChange(Number.parseInt(value, 10))
											}
										>
											<DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="auto">
												Auto
											</DropdownMenuRadioItem>
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
							<DropdownMenuSub>
								<DropdownMenuSubTrigger className="py-2.5 data-[state=open]:bg-zinc-800 focus:bg-zinc-800">
									Subtitles
								</DropdownMenuSubTrigger>
								<DropdownMenuSubContent className="z-[70] border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40">
									{subtitleTrackOptions.length === 0 ? (
										<DropdownMenuItem className="py-2.5" disabled>
											No subtitles
										</DropdownMenuItem>
									) : (
										<DropdownMenuRadioGroup
											value={selectedSubtitleTrackId?.toString() ?? "auto"}
											onValueChange={(value) => {
												if (value === "auto") {
													onSubtitleTrackChange(null);
												} else if (value === "-1") {
													onSubtitleTrackChange(-1);
												} else {
													onSubtitleTrackChange(Number.parseInt(value, 10));
												}
											}}
										>
											<DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="auto">
												Auto
											</DropdownMenuRadioItem>
											<DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="-1">
												Off
											</DropdownMenuRadioItem>
											{subtitleTrackOptions.map((track) => (
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
						</DropdownMenuContent>
					</DropdownMenu>
					<PlayerButton
						aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
						onClick={(e) => {
							e.stopPropagation();
							togglePlayerFullscreen();
						}}
					>
						{isFullscreen ? <MinimizeIcon className="size-5" /> : <MaximizeIcon className="size-5" />}
					</PlayerButton>
				</div>
			</div>
		</div>
	);
};
