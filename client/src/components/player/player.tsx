// oxlint-disable jsx_a11y/click-events-have-key-events
// oxlint-disable jsx_a11y/no-static-element-interactions
import { useCallback, useEffect, useRef, useState, type FC } from "react";
import { useClient } from "urql";
import { unmask } from "../../@generated/gql";
import { PlayerResumeDialog } from "./player-resume-dialog";
import { PlayerVideo } from "./player-video";
import {
  PlayerAudioTrack,
  PlayerQuery,
  PlayerState,
  PlayerSubtitleTrack,
  PlayerVideoTrack,
  resetPlayer,
  setPlayerStatus,
  usePlayerStore,
} from "./store/player-store";
import { PlayerBottom } from "./ui/player-bottom";
import { cn } from "../../lib/utils";
import { PlayerTop } from "./ui/player-top";
import { PlayerMiddle } from "./ui/player-middle";
import { PlayerKeybinds } from "./player-keybinds";
import { bumpPlayerControls, usePlayerControlsStore, useShowControls } from "./store/player-controls-store";
import { useVideoControls } from "./store/player-video-context";
import { PlayerSubtitleOverlay } from "./components/player-subtitle-overlay";

const PlayerOverlay: FC<{ portalContainer: HTMLElement | null }> = ({ portalContainer }) => {
  const { toggleSurfacePlaying } = useVideoControls();
  const showControls = useShowControls();
  const paused = usePlayerStore((state) => state.paused);

  return (
    <div
      className="absolute top-0 bottom-0 right-0 left-0 flex flex-col justify-between"
      onClick={(event) => {
        event.preventDefault();
        toggleSurfacePlaying();
      }}
      onDoubleClick={(event) => {
        event.preventDefault();
        event.stopPropagation();
      }}
    >
      <PlayerSubtitleOverlay />
      <div
        className={cn("transition-opacity duration-200", showControls || paused ? "opacity-100" : "opacity-0")}
        onClick={(event) => event.stopPropagation()}
        onDoubleClick={(event) => event.stopPropagation()}
      >
        <PlayerTop />
      </div>
      <PlayerMiddle />
      <div
        className={cn("transition-opacity duration-200", showControls || paused ? "opacity-100" : "opacity-0")}
        onClick={(event) => event.stopPropagation()}
        onDoubleClick={(event) => event.stopPropagation()}
      >
        <PlayerBottom portalContainer={portalContainer} />
      </div>
    </div>
  );
};

export const Player: FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [portalContainer, setPortalContainer] = useState<HTMLElement | null>(null);
  const client = useClient();
  const targetNodeId = usePlayerStore((state) => state.targetNodeId);
  const status = usePlayerStore((state) => state.status);
  const [nodeId, setNodeId] = useState<string | null>(null);
  const isFullscreen = usePlayerStore((state) => state.isFullscreen);
  const setContainerRef = useCallback((element: HTMLDivElement | null) => {
    containerRef.current = element;
    setPortalContainer(element);
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    if (isFullscreen) {
      container.requestFullscreen({ navigationUI: "hide" }).catch(() => undefined);
    } else if (document.fullscreenElement === container) {
      document.exitFullscreen().catch(() => undefined);
    }
  }, [isFullscreen]);

  useEffect(() => {
    const handleFullscreenChange = () => {
      if (!document.fullscreenElement && usePlayerStore.getState().isFullscreen) {
        usePlayerStore.setState({ isFullscreen: false });
        bumpPlayerControls();
      }
    };
    document.addEventListener("fullscreenchange", handleFullscreenChange);
    return () => document.removeEventListener("fullscreenchange", handleFullscreenChange);
  }, []);

  useEffect(() => {
    if (!targetNodeId) {
      setNodeId(null);
      return;
    }
    if (targetNodeId === nodeId) return;
    resetPlayer();
    setPlayerStatus({ state: PlayerState.Init });
    setNodeId(targetNodeId);
    client
      .query(PlayerQuery, { nodeId: targetNodeId }, { requestPolicy: "network-only" })
      .toPromise()
      .then((result) => {
        if (result.error) {
          setPlayerStatus({
            state: PlayerState.Error,
            errorMessage: "An error occured retrieving details from the server",
          });
          return;
        }

        if (!result.data) {
          setPlayerStatus({ state: PlayerState.Error, errorMessage: "This media is no longer available" });
          return;
        }

        if (!result.data.node.defaultFile) {
          setPlayerStatus({ state: PlayerState.Error, errorMessage: "Sorry, this media isn't available right now." });
          return;
        }

        if (!result.data.node.defaultFile.probe) {
          setPlayerStatus({
            state: PlayerState.Error,
            errorMessage: "Sorry, this file isn't ready to be played yet.",
          });
          return;
        }

        usePlayerStore.setState((state) => {
          state.durationSeconds = result.data!.node.defaultFile!.probe!.durationSeconds ?? 0;
          if (result.data!.node.defaultFile!.probe!.width && result.data!.node.defaultFile!.probe!.height) {
            state.aspectRatio =
              result.data!.node.defaultFile!.probe!.width / result.data!.node.defaultFile!.probe!.height;
          }
        });

        const videoTracks = unmask(PlayerVideoTrack, result.data.node.defaultFile.playback.video);
        const audioTracks = unmask(PlayerAudioTrack, result.data.node.defaultFile.playback.audio);
        const subtitleTracks = unmask(PlayerSubtitleTrack, result.data.node.defaultFile.playback.subtitles);
        const activeVideoTrack = videoTracks.find((track) => track.autoselect);
        const activeAudioTrack = audioTracks.find((track) => track.autoselect) ?? null;
        if (result.data.node.defaultFile.resumeHint) {
          setPlayerStatus({
            state: PlayerState.Resuming,
            fromTimeMs: result.data.node.defaultFile.resumeHint.startMs,
            data: result.data,
            videoTrack: activeVideoTrack!,
            videoTracks,
            audioTrack: activeAudioTrack,
            audioTracks,
            subtitleTracks,
          });
        } else {
          setPlayerStatus({
            state: PlayerState.Mounted,
            audioTrack: activeAudioTrack,
            videoTrack: activeVideoTrack!,
            audioTracks,
            videoTracks,
            subtitleTracks,
            data: result.data,
          });
        }
      });
  }, [targetNodeId]);

  if (status.state === PlayerState.Hidden) return null;

  return (
    <div
      ref={setContainerRef}
      className={cn(
        "z-50 select-none",
        isFullscreen ? "fixed left-0 right-0 top-0 bottom-0 z-50" : "fixed bottom-2 right-2",
      )}
      onPointerMove={() => bumpPlayerControls()}
      onPointerEnter={() => {
        usePlayerControlsStore.setState({ mouseIsHovering: true });
        bumpPlayerControls();
      }}
      onPointerLeave={() => {
        usePlayerControlsStore.setState({ mouseIsHovering: false });
      }}
    >
      <PlayerResumeDialog />
      <PlayerVideo>
        <PlayerKeybinds />
        <PlayerOverlay portalContainer={portalContainer} />
      </PlayerVideo>
    </div>
  );
};
