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
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState, type FC } from "react";
import { useStore } from "zustand/react";
import type { FragmentType } from "../../../@generated/gql";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { formatPlayerTime } from "../../../lib/format-player-time";
import { cn } from "../../../lib/utils";
import {
	DropdownMenu,
	DropdownMenuCheckboxItem,
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
import { usePlayerActions } from "../hooks/use-player-actions";
import { playerState, togglePlayerFullscreen } from "../player-state";
import { videoState } from "../video-state";
import { PlayerButton } from "./player-button";
import { PlayerProgressBar, PlayerTimelinePreviewSheetFragment } from "./player-progress-bar";
import { PlayerVolumeControl } from "./player-volume-control";
import { UpNextCard } from "./up-next-card";

type PlayableNode = NonNullable<NonNullable<ItemPlaybackQuery["node"]>["previousPlayable"]>;

// show the up-next card in the last 30s or 10% of the video, whichever is shorter
const PREVIEW_WINDOW_SECONDS = 30;
const PREVIEW_WINDOW_FRACTION = 0.1;
// extra countdown time after the video ends before auto-advancing
const POST_END_COUNTDOWN_SECONDS = 10;
const TICK_INTERVAL_MS = 100;

interface PlayerControlsProps {
	timelinePreviewSheets: FragmentType<typeof PlayerTimelinePreviewSheetFragment>[];
	previousPlayable: PlayableNode | null | undefined;
	nextPlayable: PlayableNode | null | undefined;
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
	previousPlayable,
	nextPlayable,
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
	const ended = useStore(videoState, (s) => s.ended);
	const upNextDismissed = useStore(videoState, (s) => s.upNextDismissed);
	const upNextCountdownCancelled = useStore(videoState, (s) => s.upNextCountdownCancelled);
	const audioTrackOptions = useStore(videoState, (s) => s.audioTrackOptions);
	const selectedAudioTrackId = useStore(videoState, (s) => s.selectedAudioTrackId);
	const subtitleTrackOptions = useStore(videoState, (s) => s.subtitleTrackOptions);
	const selectedSubtitleTrackId = useStore(videoState, (s) => s.selectedSubtitleTrackId);
	const showControls = useStore(videoState, (s) => s.showControls);
	const isSettingsMenuOpen = useStore(videoState, (s) => s.isSettingsMenuOpen);
	const { volume, isMuted, isFullscreen, autoplayNext } = useStore(playerState);
	const { onSeek, togglePlaying, onToggleMute, onVolumeChange } = usePlayerActions();
	const [hoveredButton, setHoveredButton] = useState<"previous" | "next" | null>(null);

	const hasPreviousItem = !!previousPlayable;
	const hasNextItem = !!nextPlayable;

	// up-next card state
	const previewWindowSeconds =
		duration > 0 ? Math.min(PREVIEW_WINDOW_SECONDS, duration * PREVIEW_WINDOW_FRACTION) : PREVIEW_WINDOW_SECONDS;
	const isNearEnd = duration > 0 && duration - currentTime <= previewWindowSeconds;
	const isUpNextActive = !!nextPlayable && !upNextDismissed && (isNearEnd || ended);

	// reset dismissal/cancel state when seeking out of the preview window
	const wasActiveRef = useRef(false);
	useEffect(() => {
		if (!isUpNextActive && wasActiveRef.current) {
			videoState.setState({ upNextDismissed: false, upNextCountdownCancelled: false });
		}
		wasActiveRef.current = isUpNextActive;
	}, [isUpNextActive]);

	// sync isUpNextActive into videoState so the controls-visibility hook can pin controls
	useEffect(() => {
		videoState.setState({ isUpNextActive });
	}, [isUpNextActive]);

	// countdown timer for post-end auto-advance.
	// total countdown spans from when the card appears until POST_END_COUNTDOWN_SECONDS after the video ends.
	const totalCountdownSeconds = previewWindowSeconds + POST_END_COUNTDOWN_SECONDS;
	const previewStartTime = duration - previewWindowSeconds;

	const [elapsedSinceEnd, setElapsedSinceEnd] = useState(0);
	const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

	const shouldCountdown = ended && autoplayNext && !upNextCountdownCancelled && isUpNextActive;

	useEffect(() => {
		if (!shouldCountdown) {
			setElapsedSinceEnd(0);
			if (intervalRef.current) {
				clearInterval(intervalRef.current);
				intervalRef.current = null;
			}
			return;
		}

		intervalRef.current = setInterval(() => {
			setElapsedSinceEnd((prev) => prev + TICK_INTERVAL_MS);
		}, TICK_INTERVAL_MS);

		return () => {
			if (intervalRef.current) {
				clearInterval(intervalRef.current);
				intervalRef.current = null;
			}
		};
	}, [shouldCountdown]);

	// auto-advance when countdown completes
	useEffect(() => {
		if (shouldCountdown && elapsedSinceEnd >= POST_END_COUNTDOWN_SECONDS * 1000) {
			onNextItem();
		}
	}, [shouldCountdown, elapsedSinceEnd, onNextItem]);

	// progress: fills smoothly from card appearance through to post-end countdown
	const upNextProgress = useMemo(() => {
		if (!isUpNextActive || !autoplayNext || upNextCountdownCancelled) return 0;
		if (totalCountdownSeconds <= 0) return 0;

		if (ended) {
			// video is over — progress continues from where playback left off
			const playbackPortion = previewWindowSeconds / totalCountdownSeconds;
			const postEndPortion = elapsedSinceEnd / 1000 / totalCountdownSeconds;
			return Math.min(1, playbackPortion + postEndPortion);
		}

		// still playing near end — progress based on video position
		return Math.min(1, Math.max(0, (currentTime - previewStartTime) / totalCountdownSeconds));
	}, [
		isUpNextActive,
		autoplayNext,
		upNextCountdownCancelled,
		ended,
		currentTime,
		previewStartTime,
		previewWindowSeconds,
		totalCountdownSeconds,
		elapsedSinceEnd,
	]);

	// seconds remaining until auto-advance — used for the button label
	const countdownSeconds = useMemo(() => {
		if (!isUpNextActive || !autoplayNext || upNextCountdownCancelled) return 0;
		if (ended) return Math.max(0, POST_END_COUNTDOWN_SECONDS - elapsedSinceEnd / 1000);
		return Math.max(0, duration - currentTime + POST_END_COUNTDOWN_SECONDS);
	}, [isUpNextActive, autoplayNext, upNextCountdownCancelled, ended, elapsedSinceEnd, duration, currentTime]);

	// show action buttons (Play Now / Cancel) only in the last 30s or when ended
	const showUpNextActions = isUpNextActive && (isNearEnd || ended);
	const isPreviousCardVisible = hoveredButton === "previous" && !!previousPlayable;
	const isNextCardVisible =
		!!nextPlayable && (hoveredButton === "next" || (isUpNextActive && hoveredButton !== "previous"));

	useEffect(() => {
		videoState.setState({ isItemCardOpen: isPreviousCardVisible || isNextCardVisible });
	}, [isPreviousCardVisible, isNextCardVisible]);

	useEffect(() => {
		return () => {
			videoState.setState({ isItemCardOpen: false });
		};
	}, []);

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
							<div
								className="relative overflow-visible"
								onMouseEnter={() => setHoveredButton("previous")}
								onMouseLeave={() => setHoveredButton(null)}
							>
								<AnimatePresence>
									{isPreviousCardVisible && (
										<motion.div
											key="prev-card"
											initial={{ opacity: 0, y: 8 }}
											animate={{ opacity: 1, y: 0 }}
											exit={{ opacity: 0, y: 8 }}
											transition={{ duration: 0.15 }}
											className="absolute bottom-full left-0 mb-2"
										>
											<UpNextCard
												displayName={previousPlayable.properties.displayName}
												description={previousPlayable.properties.description}
												thumbnailImage={previousPlayable.properties.thumbnailImage}
												seasonNumber={previousPlayable.properties.seasonNumber}
												episodeNumber={previousPlayable.properties.episodeNumber}
											/>
										</motion.div>
									)}
								</AnimatePresence>
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
							</div>
							<div
								className="relative overflow-visible"
								onMouseEnter={() => setHoveredButton("next")}
								onMouseLeave={() => setHoveredButton(null)}
							>
								<AnimatePresence>
									{isNextCardVisible && (
										<motion.div
											key="next-card"
											initial={{ opacity: 0, y: 8 }}
											animate={{ opacity: 1, y: 0 }}
											exit={{ opacity: 0, y: 8 }}
											transition={{ duration: 0.15 }}
											className="absolute bottom-full left-0 mb-2"
										>
											<UpNextCard
												displayName={nextPlayable.properties.displayName}
												description={nextPlayable.properties.description}
												thumbnailImage={nextPlayable.properties.thumbnailImage}
												seasonNumber={nextPlayable.properties.seasonNumber}
												episodeNumber={nextPlayable.properties.episodeNumber}
												onPlay={showUpNextActions ? onNextItem : undefined}
												onCancel={
													showUpNextActions && autoplayNext && !upNextCountdownCancelled
														? () => videoState.setState({ upNextCountdownCancelled: true })
														: undefined
												}
												progressPercent={showUpNextActions ? upNextProgress : undefined}
												countdownSeconds={showUpNextActions ? countdownSeconds : undefined}
											/>
										</motion.div>
									)}
								</AnimatePresence>
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
							</div>
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
												value === "auto" ? onAudioTrackChange(null) : onAudioTrackChange(Number.parseInt(value, 10))
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
							<DropdownMenuSeparator className="bg-zinc-700" />
							<DropdownMenuCheckboxItem
								className="py-2.5 focus:bg-zinc-800"
								checked={autoplayNext}
								onCheckedChange={(checked) => playerState.setState({ autoplayNext: !!checked })}
							>
								Autoplay
							</DropdownMenuCheckboxItem>
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
