/* oxlint-disable jsx_a11y/media-has-caption, jsx_a11y/prefer-tag-over-role */
import { useQuery } from "@apollo/client/react";
import { useEffect, useRef, type FC } from "react";
import { useStore } from "zustand/react";
import { cn } from "../../lib/utils";
import { PlayerErrorOverlay } from "./components/player-error-overlay";
import { PlayerControls } from "./components/player-controls";
import { PlayerIntroOverlay } from "./components/player-intro-overlay";
import { PlayerLoadingIndicator } from "./components/player-loading-indicator";
import { PlayerTopChrome } from "./components/player-top-chrome";
import { ResumePromptDialog } from "./components/resume-prompt-dialog";
import { useControlsVisibility } from "./hooks/use-controls-visibility";
import { useFullscreen } from "./hooks/use-fullscreen";
import { usePlaybackLifecycle } from "./hooks/use-playback-lifecycle";
import { useKeyboardShortcuts } from "./hooks/use-keyboard-shortcuts";
import { usePlayerActions } from "./hooks/use-player-actions";
import { useSurfaceInteraction } from "./hooks/use-surface-interaction";
import { useTrackSelection } from "./hooks/use-track-selection";
import { useVideoEvents } from "./hooks/use-video-events";
import { useWatchProgress } from "./hooks/use-watch-progress";
import { PlayerContext, usePlayerContext } from "./player-context";
import { PlayerLayout } from "./player-layout";
import type { PlaybackEngine } from "./engines";
import { ItemPlaybackQuery } from "./player-queries";
import { playerState, setPlayerLoading, setPlayerMedia } from "./player-state";
import { videoState } from "./video-state";

// PlayerContent runs inside PlayerContext.Provider so all hooks can access the shared refs.
const PlayerContent: FC<{ itemId: string; autoplay: boolean; shouldPromptResume: boolean }> = ({
	itemId,
	autoplay,
	shouldPromptResume,
}) => {
	const { videoRef, containerRef, surfaceRef } = usePlayerContext();
	const { isFullscreen, volume, isMuted } = useStore(playerState);
	const showControls = useStore(videoState, (s) => s.showControls);

	const {
		data,
		previousData,
		loading: isItemLoading,
		error: itemLoadError,
	} = useQuery(ItemPlaybackQuery, { variables: { itemId } });
	// keep the previous item mounted while loading so browser fullscreen is preserved
	const currentMedia = data?.node ?? (isItemLoading ? previousData?.node : null) ?? null;
	// only treat query loading as player loading while we're still resolving a different item
	const isResolvingRequestedMedia = isItemLoading && currentMedia?.id !== itemId;

	useEffect(() => {
		if (!isResolvingRequestedMedia) return;
		videoState.setState({ errorMessage: null });
		setPlayerLoading(true);
	}, [isResolvingRequestedMedia]);

	useEffect(() => {
		if (!itemLoadError) return;
		videoState.setState({ errorMessage: "Sorry, this item is unavailable" });
		setPlayerLoading(false);
	}, [itemLoadError]);

	// sync volume/mute from playerState into the video element whenever they change
	useEffect(() => {
		if (!videoRef.current) return;
		videoRef.current.volume = volume;
		videoRef.current.muted = isMuted;
	}, [volume, isMuted, videoRef]);

	usePlaybackLifecycle(currentMedia, { shouldPromptResume, autoplay });
	// useVideoEvents runs once with [] deps — the video element must be in the DOM on initial mount.
	// we always render the container + video below (never return null before them) to guarantee this.
	useVideoEvents();
	useFullscreen();
	useWatchProgress(currentMedia);

	const actions = usePlayerActions();
	const { showControlsTemporarily, beginControlsInteraction, endControlsInteraction, handleMouseLeave } =
		useControlsVisibility();
	const { handleContainerClick, handleMouseMove } = useSurfaceInteraction({
		togglePlaying: actions.togglePlaying,
		showControlsTemporarily,
	});
	const { handlePlayerKeyDown } = useKeyboardShortcuts({ actions, showControlsTemporarily, handleContainerClick });
	const { onAudioTrackChange, onSubtitleTrackChange } = useTrackSelection(currentMedia, itemId);

	// default to 16:9 until the video reports its actual dimensions
	const miniPlayerAspectRatio = Math.max(videoState.getState().videoAspectRatio, 16 / 9);

	const timelinePreviewSheets = Array.isArray(currentMedia?.file?.timelinePreview)
		? currentMedia.file.timelinePreview
		: [];

	const onPreviousItem = () => {
		const previousItemId = currentMedia?.previousPlayable?.id;
		if (previousItemId) setPlayerMedia(previousItemId, true);
	};

	const onNextItem = () => {
		const nextItemId = currentMedia?.nextPlayable?.id;
		if (nextItemId) setPlayerMedia(nextItemId, true);
	};

	// always render the container + video so videoRef.current is stable for useVideoEvents on mount.
	// the overlay and controls are gated on currentMedia being resolved.
	return (
		<div
			ref={containerRef}
			className={cn(
				isFullscreen
					? "z-50 fixed inset-0 bg-black outline-none"
					: "z-50 fixed bottom-4 right-4 rounded shadow-2xl bg-black outline-none",
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
			<video
				ref={videoRef}
				className={cn("block w-full h-full bg-black object-contain outline-none", !isFullscreen && "rounded")}
				autoPlay={autoplay}
				controls={false}
				disablePictureInPicture
			/>

			{/* Overlay surface — only rendered once media is resolved */}
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
					{/* Vignette gradient — visible when controls are shown */}
					<div
						className={cn(
							"absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/60 transition-opacity duration-300 pointer-events-none",
							showControls ? "opacity-100" : "opacity-0",
							!isFullscreen && "rounded",
						)}
					/>

					<PlayerLayout
						top={<PlayerTopChrome media={currentMedia} />}
						middle={<PlayerIntroOverlay media={currentMedia} />}
						bottom={
							<PlayerControls
								timelinePreviewSheets={timelinePreviewSheets}
								previousPlayable={currentMedia.previousPlayable}
								nextPlayable={currentMedia.nextPlayable}
								onPreviousItem={onPreviousItem}
								onNextItem={onNextItem}
								onAudioTrackChange={onAudioTrackChange}
								onSubtitleTrackChange={onSubtitleTrackChange}
								onControlsInteractionStart={beginControlsInteraction}
								onControlsInteractionEnd={endControlsInteraction}
								onControlsActivity={showControlsTemporarily}
								dropdownPortalContainer={containerRef.current}
							/>
						}
					/>
				</div>
			)}

			<ResumePromptDialog />
			<PlayerLoadingIndicator />
			<PlayerErrorOverlay />
		</div>
	);
};

// Player creates the shared refs and provides them via context so all hooks and child components
// can access them. PlayerContent lives inside the provider so usePlayerContext() resolves correctly.
export const Player: FC<{ itemId: string; autoplay?: boolean; shouldPromptResume?: boolean }> = ({
	itemId,
	autoplay = false,
	shouldPromptResume = false,
}) => {
	const videoRef = useRef<HTMLVideoElement>(null);
	const engineRef = useRef<PlaybackEngine | null>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const surfaceRef = useRef<HTMLDivElement>(null);

	return (
		<PlayerContext.Provider value={{ videoRef, engineRef, containerRef, surfaceRef }}>
			<PlayerContent itemId={itemId} autoplay={autoplay} shouldPromptResume={shouldPromptResume} />
		</PlayerContext.Provider>
	);
};
