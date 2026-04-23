/* oxlint-disable jsx_a11y/prefer-tag-over-role */
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState, type FC } from "react";
import { useQuery } from "urql";
import { client } from "../../client";
import { cn } from "../../lib/utils";
import { PlayerControls } from "./components/player-controls";
import { PlayerErrorOverlay } from "./components/player-error-overlay";
import { PlayerIntroOverlay } from "./components/player-intro-overlay";
import { PlayerLoadingIndicator } from "./components/player-loading-indicator";
import { PlayerSubtitleOverlay } from "./components/player-subtitle-overlay";
import { PlayerTopChrome } from "./components/player-top-chrome";
import { ResumePromptDialog } from "./components/resume-prompt-dialog";
import { UpNextCard } from "./components/up-next-card";
import type { PlayerController } from "./hls";
import { useControlsVisibility } from "./hooks/use-controls-visibility";
import { useFullscreen } from "./hooks/use-fullscreen";
import { useKeyboardShortcuts } from "./hooks/use-keyboard-shortcuts";
import { usePlayerActions } from "./hooks/use-player-actions";
import { useSurfaceInteraction } from "./hooks/use-surface-interaction";
import { useUpNextState } from "./hooks/use-up-next-state";
import { setPlayerControls, setPlayerState, usePlayerContext } from "./player-context";
import { PlayerLayout } from "./player-layout";
import { ItemPlaybackQuery, LeaveWatchSession } from "./player-queries";
import {
  PlayerRefsContext,
  PlayerVideoElementContext,
  usePlayerRefsContext,
  usePlayerVideoElement,
} from "./player-refs-context";
import { PlayerVideo } from "./player-video";

const PlayerContent: FC<{
  itemId: string;
  autoplay: boolean;
  shouldPromptResume: boolean;
}> = ({ itemId, autoplay, shouldPromptResume }) => {
  const { containerRef, surfaceRef } = usePlayerRefsContext();
  const videoElement = usePlayerVideoElement();
  const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
  const showControls = usePlayerContext((ctx) => ctx.controls.showControls);
  const hoveredCard = usePlayerContext((ctx) => ctx.controls.hoveredCard);
  const [videoAspectRatio, setVideoAspectRatio] = useState(16 / 9);
  const miniPlayerAspectRatio = Math.max(videoAspectRatio, 16 / 9);
  const languageHints = typeof navigator === "undefined" ? [] : [...navigator.languages];

  const [{ data, fetching: isItemLoading, error: itemLoadError }] = useQuery({
    query: ItemPlaybackQuery,
    variables: { itemId, languageHints },
  });
  const currentMedia = data?.node ?? null;
  const isResolvingRequestedMedia = isItemLoading && currentMedia?.id !== itemId;

  useEffect(() => {
    if (!videoElement) {
      setVideoAspectRatio(16 / 9);
      return;
    }

    const syncAspectRatio = () => {
      if (videoElement.videoWidth <= 0 || videoElement.videoHeight <= 0) return;
      setVideoAspectRatio(videoElement.videoWidth / videoElement.videoHeight);
    };

    syncAspectRatio();
    videoElement.addEventListener("loadedmetadata", syncAspectRatio);
    videoElement.addEventListener("resize", syncAspectRatio);

    return () => {
      videoElement.removeEventListener("loadedmetadata", syncAspectRatio);
      videoElement.removeEventListener("resize", syncAspectRatio);
    };
  }, [videoElement]);

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
  const { switchItem } = actions;

  const onPreviousItem = () => {
    const previousItemId = currentMedia?.previousPlayable?.id;
    if (previousItemId) switchItem(previousItemId);
  };

  const onNextItem = () => {
    const nextItemId = currentMedia?.nextPlayable?.id;
    if (nextItemId) switchItem(nextItemId);
  };

  const upNextState = useUpNextState({ hasNextItem: !!currentMedia?.nextPlayable, onNextItem });
  const showPreviousCard = hoveredCard === "previous" && !!currentMedia?.previousPlayable;
  const showNextPreview = hoveredCard === "next" && !!currentMedia?.nextPlayable;
  const showUpNextCard = isFullscreen && upNextState.isUpNextActive && !!currentMedia?.nextPlayable;
  const cardNode = showPreviousCard ? currentMedia?.previousPlayable : currentMedia?.nextPlayable;
  const cardVisible = showPreviousCard || showNextPreview || showUpNextCard;
  const timelinePreviewSheets = Array.isArray(currentMedia?.defaultFile?.timelinePreview)
    ? currentMedia.defaultFile.timelinePreview
    : [];

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

      {currentMedia ? (
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
              "pointer-events-none absolute inset-0 transition-opacity duration-300",
              isFullscreen ? (showControls ? "opacity-100" : "opacity-0") : "opacity-0 group-hover/player:opacity-100",
              !isFullscreen && "rounded",
            )}
          />

          <PlayerLayout
            top={<PlayerTopChrome media={currentMedia} />}
            middle={
              <>
                <PlayerSubtitleOverlay />
                <PlayerIntroOverlay media={currentMedia} />
              </>
            }
            bottom={
              isFullscreen ? (
                <div className="relative z-10">
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
      ) : (
        <div className="absolute inset-0">
          <PlayerLayout top={<PlayerTopChrome media={null} />} middle={null} bottom={null} />
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
  const controllerRef = useRef<PlayerController | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const surfaceRef = useRef<HTMLDivElement>(null);
  const [videoElement, setVideoElement] = useState<HTMLVideoElement | null>(null);
  const watchSession = usePlayerContext((ctx) => ctx.watchSession);

  useEffect(() => {
    setPlayerControls({ showControls: true });
  }, [itemId]);

  useEffect(() => {
    return () => {
      const sessionId = watchSession.sessionId;
      const playerId = watchSession.playerId;
      if (!sessionId || !playerId) return;
      void client
        .mutation(LeaveWatchSession, {
          sessionId,
          playerId,
        })
        .toPromise();
    };
  }, [watchSession.playerId, watchSession.sessionId]);

  return (
    <PlayerRefsContext.Provider value={{ controllerRef, containerRef, surfaceRef }}>
      <PlayerVideoElementContext.Provider value={{ videoElement, setVideoElement }}>
        <PlayerContent itemId={itemId} autoplay={autoplay} shouldPromptResume={shouldPromptResume} />
      </PlayerVideoElementContext.Provider>
    </PlayerRefsContext.Provider>
  );
};
