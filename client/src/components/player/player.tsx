/* oxlint-disable jsx_a11y/prefer-tag-over-role */
import { useQuery } from "@apollo/client/react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, type FC } from "react";
import { cn } from "../../lib/utils";
import { PlayerErrorOverlay } from "./components/player-error-overlay";
import { PlayerControls } from "./components/player-controls";
import { PlayerIntroOverlay } from "./components/player-intro-overlay";
import { PlayerLoadingIndicator } from "./components/player-loading-indicator";
import { PlayerTopChrome } from "./components/player-top-chrome";
import { ResumePromptDialog } from "./components/resume-prompt-dialog";
import { UpNextCard } from "./components/up-next-card";
import type { PlayerController } from "./hls";
import { useControlsVisibility } from "./hooks/use-controls-visibility";
import { useFullscreen } from "./hooks/use-fullscreen";
import { useKeyboardShortcuts } from "./hooks/use-keyboard-shortcuts";
import { usePlayerActions } from "./hooks/use-player-actions";
import { useSurfaceInteraction } from "./hooks/use-surface-interaction";
import { useTrackSelection } from "./hooks/use-track-selection";
import { useUpNextState } from "./hooks/use-up-next-state";
import { setPlayerControls, setPlayerMedia, setPlayerState, usePlayerContext } from "./player-context";
import { PlayerLayout } from "./player-layout";
import { ItemPlaybackQuery } from "./player-queries";
import { PlayerRefsContext, usePlayerRefsContext } from "./player-refs-context";
import { PlayerVideo, getTimelinePreviewSheets } from "./player-video";

const PlayerContent: FC<{ itemId: string; autoplay: boolean; shouldPromptResume: boolean }> = ({
	itemId,
	autoplay,
	shouldPromptResume,
}) => {
	const { containerRef, surfaceRef } = usePlayerRefsContext();
	const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
	const showControls = usePlayerContext((ctx) => ctx.controls.showControls);
	const hoveredCard = usePlayerContext((ctx) => ctx.controls.hoveredCard);
	const miniPlayerAspectRatio = usePlayerContext((ctx) => Math.max(ctx.state.videoAspectRatio, 16 / 9));

	const {
		data,
		previousData,
		loading: isItemLoading,
		error: itemLoadError,
	} = useQuery(ItemPlaybackQuery, { variables: { itemId } });

	const currentMedia = data?.node ?? (isItemLoading ? previousData?.node : null) ?? null;
	const isResolvingRequestedMedia = isItemLoading && currentMedia?.id !== itemId;

	useEffect(() => {
		if (!isResolvingRequestedMedia) return;
		setPlayerState({ errorMessage: null, isLoading: true });
	}, [isResolvingRequestedMedia]);

	useEffect(() => {
		if (!itemLoadError) return;
		setPlayerState({ errorMessage: "Sorry, this item is unavailable", isLoading: false });
	}, [itemLoadError]);

	useFullscreen();
	const actions = usePlayerActions();
	const { showControlsTemporarily, handleMouseLeave } = useControlsVisibility();
	const { handleContainerClick, handleMouseMove } = useSurfaceInteraction({
		togglePlaying: actions.togglePlaying,
		showControlsTemporarily,
	});
	const { handlePlayerKeyDown } = useKeyboardShortcuts({ actions, handleContainerClick });
	const { onAudioTrackChange, onSubtitleTrackChange } = useTrackSelection(currentMedia, itemId);

	const onPreviousItem = () => {
		const previousItemId = currentMedia?.previousPlayable?.id;
		if (previousItemId) setPlayerMedia(previousItemId, true);
	};

	const onNextItem = () => {
		const nextItemId = currentMedia?.nextPlayable?.id;
		if (nextItemId) setPlayerMedia(nextItemId, true);
	};

	const upNextState = useUpNextState({ hasNextItem: !!currentMedia?.nextPlayable, onNextItem });
	const showPreviousCard = hoveredCard === "previous" && !!currentMedia?.previousPlayable;
	const showNextPreview = hoveredCard === "next" && !!currentMedia?.nextPlayable;
	const showUpNextCard = isFullscreen && upNextState.isUpNextActive && !!currentMedia?.nextPlayable;
	const cardNode = showPreviousCard ? currentMedia?.previousPlayable : currentMedia?.nextPlayable;
	const cardVisible = showPreviousCard || showNextPreview || showUpNextCard;
	const timelinePreviewSheets = getTimelinePreviewSheets(currentMedia);

	const cardElement =
		cardVisible && cardNode ? (
			<motion.div
				key={showPreviousCard ? "prev-card" : showNextPreview ? "next-card" : "up-next-card"}
				initial={{ opacity: 0, translateX: -12 }}
				animate={{ opacity: 1, translateX: 0 }}
				exit={{ opacity: 0, translateX: -12 }}
				transition={{ duration: 0.1 }}
			>
				<UpNextCard
					displayName={cardNode.properties.displayName}
					description={cardNode.properties.description}
					thumbnailImage={cardNode.properties.thumbnailImage}
					seasonNumber={cardNode.properties.seasonNumber}
					episodeNumber={cardNode.properties.episodeNumber}
					onPlay={showUpNextCard ? onNextItem : undefined}
					onCancel={
						showUpNextCard
							? () =>
									setPlayerState({
										upNextDismissed: true,
										upNextCountdownCancelled: true,
									})
							: undefined
					}
					progressPercent={showUpNextCard ? upNextState.upNextProgress : undefined}
					countdownSeconds={showUpNextCard ? upNextState.countdownSeconds : undefined}
				/>
			</motion.div>
		) : null;

	const controls = currentMedia ? (
		<PlayerControls
			mode={isFullscreen ? "fullscreen" : "mini"}
			timelinePreviewSheets={timelinePreviewSheets}
			previousPlayable={currentMedia.previousPlayable}
			nextPlayable={currentMedia.nextPlayable}
			onPreviousItem={onPreviousItem}
			onNextItem={onNextItem}
			onAudioTrackChange={onAudioTrackChange}
			onSubtitleTrackChange={onSubtitleTrackChange}
			dropdownPortalContainer={containerRef.current}
		/>
	) : null;

	const playerDiv = (
		<div
			ref={containerRef}
			className={cn(
				isFullscreen
					? "fixed inset-0 z-50 bg-black outline-none"
					: "group/player relative rounded bg-black shadow-2xl outline-none",
			)}
			style={
				isFullscreen
					? undefined
					: {
							aspectRatio: miniPlayerAspectRatio,
							width: `min(80dvw, max(32rem, calc(18rem * ${miniPlayerAspectRatio})))`,
						}
			}
			onMouseMove={handleMouseMove}
			onMouseLeave={handleMouseLeave}
		>
			<PlayerVideo currentMedia={currentMedia} autoplay={autoplay} shouldPromptResume={shouldPromptResume} />

			{currentMedia && (
				<div
					ref={surfaceRef}
					className={cn(
						"absolute inset-0 cursor-pointer select-none outline-none focus:outline-none focus-visible:outline-none focus-visible:ring-0",
						!isFullscreen && "rounded",
					)}
					role="button"
					tabIndex={0}
					onKeyDown={handlePlayerKeyDown}
					onMouseDownCapture={(event) => {
						const target = event.target as HTMLElement | null;
						if (target?.closest("button, [role='slider']")) return;
						surfaceRef.current?.focus();
					}}
					onClick={handleContainerClick}
					aria-label="Toggle play/pause"
				>
					<div
						className={cn(
							"pointer-events-none absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/60 transition-opacity duration-300",
							isFullscreen ? (showControls ? "opacity-100" : "opacity-0") : "opacity-0 group-hover/player:opacity-100",
							!isFullscreen && "rounded",
						)}
					/>

					<PlayerLayout
						top={<PlayerTopChrome media={currentMedia} />}
						middle={<PlayerIntroOverlay media={currentMedia} />}
						bottom={
							isFullscreen ? (
								<div className="relative">
									<div className="pointer-events-auto absolute bottom-36 left-4">
										<AnimatePresence mode="wait">{cardElement}</AnimatePresence>
									</div>
									{controls}
								</div>
							) : (
								controls
							)
						}
					/>
				</div>
			)}

			<ResumePromptDialog />
			<PlayerLoadingIndicator />
			<PlayerErrorOverlay />
		</div>
	);

	return <div className={cn(!isFullscreen && "fixed bottom-4 right-4 z-50")}>{playerDiv}</div>;
};

export const Player: FC<{ itemId: string; autoplay?: boolean; shouldPromptResume?: boolean }> = ({
	itemId,
	autoplay = false,
	shouldPromptResume = false,
}) => {
	const videoRef = useRef<HTMLVideoElement>(null);
	const controllerRef = useRef<PlayerController | null>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const surfaceRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		setPlayerControls({ showControls: true });
	}, [itemId]);

	return (
		<PlayerRefsContext.Provider value={{ videoRef, controllerRef, containerRef, surfaceRef }}>
			<PlayerContent itemId={itemId} autoplay={autoplay} shouldPromptResume={shouldPromptResume} />
		</PlayerRefsContext.Provider>
	);
};
