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
        if (!video && session.mode !== "SYNCED") return;

        const positionMs = Math.max(0, Math.round((video?.currentTime ?? playerRuntimeStore.getState().targetTime ?? 0) * 1000));
        if (session.mode === "SYNCED") {
          await sendAction(video?.paused ? WatchSessionActionKind.Play : WatchSessionActionKind.Pause, { positionMs });
          return;
        }

        if (!video) return;
        if (video.paused) {
          await video.play().catch(() => undefined);
          void sendAction(WatchSessionActionKind.Play, { positionMs }).catch((error) =>
            console.error("failed to send play action", error),
          );
        } else {
          video.pause();
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
