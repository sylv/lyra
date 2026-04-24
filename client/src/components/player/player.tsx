import { useEffect, useRef, type FC, type RefObject } from "react";
import { useQuery } from "urql";
import { graphql } from "../../@generated/gql";
import { cn } from "../../lib/utils";
import { PlayerBottomBar } from "./components/player-bottom-bar";
import { PlayerErrorOverlay } from "./components/player-error-overlay";
import { PlayerLoadingIndicator } from "./components/player-loading-indicator";
import { PlayerMiddle } from "./components/player-middle";
import { PlayerOverlayLayout } from "./components/player-overlay-layout";
import { PlayerResumePrompt } from "./components/player-resume-prompt";
import { PlayerSkipIntroOverlay } from "./components/player-skip-intro-overlay";
import { PlayerSubtitleOverlay } from "./components/player-subtitle-overlay";
import { PlayerSurface } from "./components/player-surface";
import { PlayerTopBar } from "./components/player-top-bar";
import { PlayerUpNextOverlay } from "./components/player-up-next-overlay";
import { usePlayerDisplayState } from "./hooks/use-player-display-state";
import { PlayerResumePromptProvider } from "./player-resume-prompt-state";
import { setPlayerRuntimeState, togglePlayerFullscreen, usePlayerRuntimeStore } from "./player-runtime-store";
import { PlayerSession } from "./player-session";
import { PlayerVideoProvider } from "./player-video-context";
import { PlayerVisibilityProvider } from "./player-visibility";
import { PlayerVideo } from "./player-video";

const ItemPlaybackQuery = graphql(`
  query ItemPlayback($itemId: String!, $languageHints: [String!]) {
    node(nodeId: $itemId) {
      id
      ...PlayerMetadata
      ...PlayerSkipIntro
      ...PlayerUpNext
      defaultFile {
        id
        probe {
          runtimeMinutes
        }
        playbackOptions(languageHints: $languageHints) {
          videoRenditions {
            renditionId
            displayName
            displayInfo
            codecTag
            onDemand
          }
          audioTracks {
            streamIndex
            displayName
            language
            recommended
            renditions {
              renditionId
              codecName
              bitrate
              channels
              sampleRate
              codecTag
              onDemand
            }
          }
          subtitleTracks {
            id
            streamIndex
            displayName
            languageBcp47
            flags
            autoselect
            renditions {
              id
              codecName
              type
              displayInfo
              onDemand
            }
          }
        }
        timelinePreview {
          ...PlayerTimelinePreviewSheet
        }
      }
      previousPlayable {
        id
        ...PlayerNavigation
      }
      nextPlayable {
        id
        ...PlayerNavigation
      }
      watchProgress {
        id
        progressPercent
        completed
        updatedAt
      }
    }
  }
`);

const useFullscreen = (containerRef: RefObject<HTMLDivElement | null>, isFullscreen: boolean) => {
  useEffect(() => {
    if (!containerRef.current) return;
    if (isFullscreen) {
      containerRef.current.requestFullscreen({ navigationUI: "hide" }).catch(() => false);
    } else if (document.fullscreenElement) {
      document.exitFullscreen().catch(() => false);
    }
  }, [containerRef, isFullscreen]);

  useEffect(() => {
    const handleFullscreenChange = () => {
      if (!document.fullscreenElement) {
        togglePlayerFullscreen(false);
      }
    };
    document.addEventListener("fullscreenchange", handleFullscreenChange);
    return () => document.removeEventListener("fullscreenchange", handleFullscreenChange);
  }, []);
};

export const Player: FC<{ itemId: string }> = ({ itemId }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);
  const aspectRatio = usePlayerRuntimeStore((state) => state.aspectRatio);
  const languageHints = typeof navigator === "undefined" ? [] : [...navigator.languages];
  const [{ data, fetching, error }] = useQuery({
    query: ItemPlaybackQuery,
    variables: { itemId, languageHints },
  });
  const media = data?.node ?? null;
  const { currentTime, duration } = usePlayerDisplayState(media);
  const miniPlayerAspectRatio = Math.max(aspectRatio, 16 / 9);

  useFullscreen(containerRef, isFullscreen);

  useEffect(() => {
    if (fetching && media?.id !== itemId) {
      setPlayerRuntimeState({ buffering: true, errorMessage: null });
    }
  }, [fetching, itemId, media?.id]);

  useEffect(() => {
    if (!error) return;
    setPlayerRuntimeState({
      errorMessage: "Sorry, this item is unavailable",
      buffering: false,
    });
  }, [error]);

  return (
    <PlayerVideoProvider>
      <PlayerResumePromptProvider>
        <PlayerVisibilityProvider>
          <PlayerSession media={media}>
            <div className={cn(!isFullscreen && "fixed bottom-4 right-4 z-50")}>
              <div
                ref={containerRef}
                className={cn(
                  isFullscreen ? "fixed inset-0 z-50 bg-black outline-none" : "relative rounded bg-black shadow-2xl outline-none",
                )}
                style={
                  isFullscreen
                    ? undefined
                    : {
                        aspectRatio: miniPlayerAspectRatio,
                        width: `min(80dvw, max(32rem, calc(18rem * ${miniPlayerAspectRatio})))`,
                      }
                }
              >
                <PlayerVideo media={media} />
                <PlayerSurface fullscreen={isFullscreen}>
                  <PlayerOverlayLayout
                    top={<PlayerTopBar media={media} portalContainer={containerRef.current} />}
                    middle={
                      <PlayerMiddle>
                        <PlayerSubtitleOverlay media={media} />
                        {media ? <PlayerSkipIntroOverlay media={media} /> : null}
                        {media ? <PlayerUpNextOverlay media={media} /> : null}
                      </PlayerMiddle>
                    }
                    bottom={
                      media ? (
                        <PlayerBottomBar
                          compact={!isFullscreen}
                          currentTime={currentTime}
                          duration={duration}
                          previousPlayable={media.previousPlayable}
                          nextPlayable={media.nextPlayable}
                          timelinePreviewSheets={Array.isArray(media.defaultFile?.timelinePreview) ? media.defaultFile.timelinePreview : []}
                          portalContainer={containerRef.current}
                        />
                      ) : null
                    }
                  />
                </PlayerSurface>
                <PlayerResumePrompt portalContainer={containerRef.current} />
                <PlayerLoadingIndicator />
                <PlayerErrorOverlay />
              </div>
            </div>
          </PlayerSession>
        </PlayerVisibilityProvider>
      </PlayerResumePromptProvider>
    </PlayerVideoProvider>
  );
};
