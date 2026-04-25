import { useMemo } from "react";
import { WatchSessionActionKind } from "../../../@generated/gql/graphql";
import { playerOptionsStore } from "../player-options-store";
import { playerRuntimeStore, setPlayerMedia, setPlayerRuntimeState } from "../player-runtime-store";
import { usePlayerSession } from "../player-session";
import { usePlayerVideoElement } from "../player-video-context";

const clampTargetTime = (time: number, duration: number) => {
  const safeTime = Math.max(0, time);
  if (!Number.isFinite(duration) || duration <= 0) return safeTime;
  return Math.min(duration, safeTime);
};

const resolveToggleAction = (video: HTMLVideoElement | null) => {
  const runtime = playerRuntimeStore.getState();
  if (runtime.ended) return "play" as const;

  const startupPending = runtime.autoplay && !runtime.hasMediaLoaded;
  if (!video) return startupPending ? ("pause" as const) : ("play" as const);
  if (!video.paused) return "pause" as const;
  if (startupPending) return "pause" as const;
  return "play" as const;
};

export const usePlayerCommands = () => {
  const videoElement = usePlayerVideoElement();
  const { session, sendAction } = usePlayerSession();

  const seekTo = async (time: number) => {
    const runtime = playerRuntimeStore.getState();
    const video = videoElement;
    const currentDuration =
      (video && Number.isFinite(video.duration) && video.duration > 0 ? video.duration : runtime.duration) ?? 0;
    const target = clampTargetTime(time, currentDuration);
    const positionMs = Math.round(target * 1000);

    setPlayerRuntimeState({
      targetTime: target,
      currentTime: target,
      ended: false,
    });

    if (session.mode === "SYNCED") {
      await sendAction(WatchSessionActionKind.Seek, { positionMs });
      return;
    }

    if (video) {
      video.currentTime = target;
    }
    void sendAction(WatchSessionActionKind.Seek, { positionMs }).catch((error) =>
      console.error("failed to send seek action", error),
    );
  };

  return useMemo(
    () => ({
      async togglePlaying() {
        const video = videoElement;
        const runtime = playerRuntimeStore.getState();
        const nextAction = resolveToggleAction(video);
        const positionMs = Math.max(
          0,
          Math.round((video?.currentTime ?? runtime.targetTime ?? runtime.currentTime ?? 0) * 1000),
        );
        if (session.mode === "SYNCED") {
          await sendAction(nextAction === "play" ? WatchSessionActionKind.Play : WatchSessionActionKind.Pause, {
            positionMs,
          });
          return;
        }

        if (nextAction === "play") {
          setPlayerRuntimeState({ autoplay: true, ended: false, errorMessage: null });
          await video?.play().catch(() => undefined);
          void sendAction(WatchSessionActionKind.Play, { positionMs }).catch((error) =>
            console.error("failed to send play action", error),
          );
        } else {
          setPlayerRuntimeState({ autoplay: false, playing: false, buffering: false });
          video?.pause();
          void sendAction(WatchSessionActionKind.Pause, { positionMs }).catch((error) =>
            console.error("failed to send pause action", error),
          );
        }
      },
      seekTo,
      async seekBy(deltaSeconds: number) {
        const runtime = playerRuntimeStore.getState();
        const video = videoElement;
        const currentTime = video?.currentTime ?? runtime.targetTime ?? runtime.currentTime;
        const currentDuration =
          (video && Number.isFinite(video.duration) && video.duration > 0 ? video.duration : runtime.duration) ?? 0;
        const target = clampTargetTime(currentTime + deltaSeconds, currentDuration);
        await seekTo(target);
      },
      toggleMute() {
        const options = playerOptionsStore.getState();
        const nextMuted = !options.isMuted;
        options.setMuted(nextMuted);
        if (videoElement) {
          videoElement.muted = nextMuted;
        }
      },
      setVolume(volume: number) {
        const options = playerOptionsStore.getState();
        options.setVolume(volume);
        if (videoElement) {
          videoElement.volume = volume;
          if (volume > 0 && options.isMuted) {
            options.setMuted(false);
            videoElement.muted = false;
          }
        } else if (volume > 0 && options.isMuted) {
          options.setMuted(false);
        }
      },
      setAudioTrack(trackId: number | null) {
        setPlayerRuntimeState({ selectedAudioTrackId: trackId });
      },
      setSubtitleTrack(trackId: string | null) {
        setPlayerRuntimeState({ selectedSubtitleTrackId: trackId });
      },
      setVideoRendition(renditionId: string | null) {
        setPlayerRuntimeState({ selectedVideoRenditionId: renditionId });
      },
      async switchItem(itemId: string) {
        if (session.mode === "SYNCED") {
          await sendAction(WatchSessionActionKind.SwitchItem, { nodeId: itemId });
          return;
        }

        setPlayerMedia(itemId, true);
        void sendAction(WatchSessionActionKind.SwitchItem, { nodeId: itemId }).catch((error) =>
          console.error("failed to send switch item action", error),
        );
      },
    }),
    [seekTo, sendAction, session.mode, videoElement],
  );
};
