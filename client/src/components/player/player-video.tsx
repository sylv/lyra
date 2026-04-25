/* oxlint-disable jsx_a11y/media-has-caption */
import { useEffect, useMemo, useRef, useState, type FC } from "react";
import { useMutation } from "urql";
import { graphql } from "../../@generated/gql";
import { type ItemPlaybackQuery } from "../../@generated/gql/graphql";
import { createHlsPlayer, type PlayerController } from "./hls";
import { playerOptionsStore } from "./player-options-store";
import { playerRuntimeStore, setPlayerRuntimeState, usePlayerRuntimeStore } from "./player-runtime-store";
import { usePlayerResumePrompt } from "./player-resume-prompt-state";
import { usePlayerSession } from "./player-session";
import { usePlayerVideoRegistration } from "./player-video-context";
import { pickPreferredSubtitleRendition } from "./subtitles";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;
type PlaybackVideoRendition = NonNullable<
  NonNullable<NonNullable<CurrentMedia["defaultFile"]>["playbackOptions"]>["videoRenditions"][number]
>;

const MintPlaybackUrl = graphql(`
  mutation MintPlaybackUrl($input: PlaybackUrlInput!) {
    mintPlaybackUrl(input: $input) {
      url
      packagerId
    }
  }
`);

const UpdateWatchState = graphql(`
  mutation UpdateWatchState($fileId: String!, $progressPercent: Float!) {
    updateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {
      progressPercent
      updatedAt
    }
  }
`);

const isVideoRenditionPlayable = (rendition: PlaybackVideoRendition, probe: HTMLVideoElement) => {
  const mimeType = `video/mp4; codecs="${rendition.codecTag}"`;
  const support = probe.canPlayType(mimeType);
  return support === "probably" || support === "maybe";
};

const listPreferredVideoRenditions = (
  renditions:
    | NonNullable<NonNullable<NonNullable<CurrentMedia["defaultFile"]>["playbackOptions"]>["videoRenditions"]>
    | null
    | undefined,
) => {
  if (!renditions || renditions.length === 0) return null;
  const probe = document.createElement("video");
  const playable = renditions.filter((rendition) => isVideoRenditionPlayable(rendition, probe));
  return playable.length > 0 ? playable : renditions;
};

const pickPlayableVideoRendition = (
  renditions: PlaybackVideoRendition[] | null,
  selectedRenditionId: string | null,
) => {
  if (!renditions || renditions.length === 0) return null;
  if (selectedRenditionId != null) {
    const selected = renditions.find((rendition) => rendition.renditionId === selectedRenditionId);
    if (selected) return selected;
  }
  return renditions[0];
};

const resolveResumePositionSeconds = (
  watchProgressPercent: number | null | undefined,
  runtimeDurationSeconds: number | null,
) => {
  if (
    typeof watchProgressPercent !== "number" ||
    !Number.isFinite(watchProgressPercent) ||
    watchProgressPercent <= 0 ||
    watchProgressPercent >= 1 ||
    typeof runtimeDurationSeconds !== "number" ||
    !Number.isFinite(runtimeDurationSeconds) ||
    runtimeDurationSeconds <= 0
  ) {
    return null;
  }

  const progress = Math.max(0, Math.min(0.999, watchProgressPercent));
  const maxStart = Math.max(0, runtimeDurationSeconds - 0.5);
  return Math.max(0, Math.min(progress * runtimeDurationSeconds, maxStart));
};

export const PlayerVideo: FC<{ media: CurrentMedia | null }> = ({ media }) => {
  const setVideoElement = usePlayerVideoRegistration();
  const { session } = usePlayerSession();
  const { openPrompt } = usePlayerResumePrompt();
  const videoRef = useRef<HTMLVideoElement>(null);
  const controllerRef = useRef<PlayerController | null>(null);
  const [, mintPlaybackUrl] = useMutation(MintPlaybackUrl);
  const [, updateWatchProgress] = useMutation(UpdateWatchState);
  const snapshotUpdateRef = useRef<{ mediaId: string | null; lastPosition: number | null; lastUpdatedAt: number }>({
    mediaId: null,
    lastPosition: null,
    lastUpdatedAt: 0,
  });
  const watchProgressRef = useRef<{
    mediaId: string | null;
    fileId: string | null;
    lastProgressPercent: number | null;
  }>({
    mediaId: null,
    fileId: null,
    lastProgressPercent: null,
  });
  const initialWatchProgressRef = useRef<{
    loadKey: string;
    completed: boolean | null;
    progressPercent: number | null;
  } | null>(null);
  const autoplay = usePlayerRuntimeStore((state) => state.autoplay);
  const shouldPromptResume = usePlayerRuntimeStore((state) => state.shouldPromptResume);
  const selectedVideoRenditionId = usePlayerRuntimeStore((state) => state.selectedVideoRenditionId);
  const selectedAudioTrackId = usePlayerRuntimeStore((state) => state.selectedAudioTrackId);
  const selectedSubtitleTrackId = usePlayerRuntimeStore((state) => state.selectedSubtitleTrackId);
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);
  const playbackOptions = media?.defaultFile?.playbackOptions ?? null;
  const subtitleTracks = playbackOptions?.subtitleTracks ?? [];
  const preferredVideoRenditions = listPreferredVideoRenditions(playbackOptions?.videoRenditions);
  const recommendedAudioTrack =
    playbackOptions?.audioTracks.find((track) => track.recommended) ?? playbackOptions?.audioTracks[0] ?? null;
  const activeAudioTrack =
    playbackOptions?.audioTracks.find((track) => track.streamIndex === selectedAudioTrackId) ?? recommendedAudioTrack;
  const activeAudioRendition = activeAudioTrack?.renditions[0] ?? null;
  const activeVideoRendition = pickPlayableVideoRendition(preferredVideoRenditions, selectedVideoRenditionId);
  const defaultFileId = media?.defaultFile?.id ?? null;
  const runtimeMinutes = media?.defaultFile?.probe?.runtimeMinutes ?? null;
  const watchProgressCompleted = media?.watchProgress?.completed ?? null;
  const watchProgressPercent = media?.watchProgress?.progressPercent ?? null;
  const shouldPromptResumeRef = useRef(false);
  const [resumeSelection, setResumeSelection] = useState<{ loadKey: string; startTime: number | null } | null>(null);
  const loadKey = `${media?.id ?? ""}:${defaultFileId ?? ""}:${activeVideoRendition?.renditionId ?? ""}:${activeAudioTrack?.streamIndex ?? ""}:${activeAudioRendition?.renditionId ?? ""}`;

  useEffect(() => {
    initialWatchProgressRef.current = {
      loadKey,
      completed: watchProgressCompleted,
      progressPercent: watchProgressPercent,
    };
  }, [loadKey]);

  useEffect(() => {
    if (shouldPromptResume) {
      shouldPromptResumeRef.current = true;
    }
  }, [shouldPromptResume, media?.id]);

  useEffect(() => {
    setResumeSelection(null);
  }, [loadKey]);

  useEffect(() => {
    setVideoElement(videoRef.current);
    return () => setVideoElement(null);
  }, [setVideoElement]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    const options = playerOptionsStore.getState();
    video.volume = options.volume;
    video.muted = options.isMuted;
  }, []);

  useEffect(() => {
    setPlayerRuntimeState({
      videoRenditionOptions:
        preferredVideoRenditions?.map((rendition) => ({
          id: rendition.renditionId,
          label: rendition.displayName,
          displayInfo: rendition.displayInfo,
          onDemand: rendition.onDemand,
        })) ?? [],
      audioTrackOptions:
        playbackOptions?.audioTracks.map((track) => ({
          id: track.streamIndex,
          label: track.displayName,
          language: track.language ?? null,
        })) ?? [],
      subtitleTrackOptions:
        subtitleTracks
          .map((track) => {
            const rendition = pickPreferredSubtitleRendition(track);
            if (!rendition) return null;
            return {
              id: track.id,
              label: track.displayName,
              language: track.languageBcp47 ?? null,
              flags: track.flags,
              autoselect: track.autoselect,
              renditionId: rendition.id,
              renditionType: rendition.type,
              displayInfo: rendition.displayInfo,
              onDemand: rendition.onDemand,
            };
          })
          .filter((track): track is NonNullable<typeof track> => track != null) ?? [],
    });
  }, [playbackOptions?.audioTracks, preferredVideoRenditions, subtitleTracks]);

  useEffect(() => {
    setPlayerRuntimeState({
      selectedVideoRenditionId: null,
      selectedAudioTrackId: null,
      selectedSubtitleTrackId: null,
      activeSubtitleTrackId: null,
      activeSubtitleRenditionId: null,
      pendingSubtitleTrackId: null,
    });
  }, [defaultFileId]);

  useEffect(() => {
    const hasSelectedAudioTrack =
      selectedAudioTrackId == null ||
      playbackOptions?.audioTracks.some((track) => track.streamIndex === selectedAudioTrackId);
    if (!hasSelectedAudioTrack) {
      setPlayerRuntimeState({ selectedAudioTrackId: null });
    }
  }, [playbackOptions?.audioTracks, selectedAudioTrackId]);

  useEffect(() => {
    if (selectedVideoRenditionId == null) return;
    const hasSelectedVideoRendition = preferredVideoRenditions?.some(
      (rendition) => rendition.renditionId === selectedVideoRenditionId,
    );
    if (!hasSelectedVideoRendition) {
      setPlayerRuntimeState({ selectedVideoRenditionId: null });
    }
  }, [preferredVideoRenditions, selectedVideoRenditionId]);

  useEffect(() => {
    if (selectedSubtitleTrackId == null || selectedSubtitleTrackId === "") return;
    const hasSelectedSubtitleTrack = playbackOptions?.subtitleTracks.some(
      (track) => track.id === selectedSubtitleTrackId,
    );
    if (!hasSelectedSubtitleTrack) {
      setPlayerRuntimeState({ selectedSubtitleTrackId: null });
    }
  }, [playbackOptions?.subtitleTracks, selectedSubtitleTrackId]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    controllerRef.current?.destroy();
    controllerRef.current = null;

    if (!media?.defaultFile) {
      video.pause();
      video.removeAttribute("src");
      video.load();
      if (media) {
        setPlayerRuntimeState({
          errorMessage: "Sorry, this item is unavailable",
          buffering: false,
        });
      }
      return;
    }

    if (!session.playerId) return;
    if (!activeAudioTrack || !activeAudioRendition || !activeVideoRendition) {
      setPlayerRuntimeState({
        errorMessage: "Sorry, this item has no playable stream",
        buffering: false,
      });
      return;
    }

    const runtime = playerRuntimeStore.getState();
    const startupTargetTime =
      !runtime.hasMediaLoaded &&
      typeof runtime.targetTime === "number" &&
      Number.isFinite(runtime.targetTime) &&
      runtime.targetTime >= 0
        ? runtime.targetTime
        : null;
    const livePlaybackPositionSeconds =
      runtime.currentItemId === media.id && (runtime.hasMediaLoaded || controllerRef.current != null)
        ? Number.isFinite(video.currentTime) && video.currentTime > 0
          ? video.currentTime
          : Number.isFinite(runtime.currentTime) && runtime.currentTime > 0
            ? runtime.currentTime
            : null
        : null;

    const runtimeDurationSeconds =
      typeof runtimeMinutes === "number" && Number.isFinite(runtimeMinutes) && runtimeMinutes > 0
        ? runtimeMinutes * 60
        : null;
    const initialWatchProgress =
      initialWatchProgressRef.current?.loadKey === loadKey
        ? initialWatchProgressRef.current
        : {
            completed: watchProgressCompleted,
            progressPercent: watchProgressPercent,
          };
    const effectiveWatchProgressPercent = initialWatchProgress.completed ? null : initialWatchProgress.progressPercent;
    const watchProgressStartTime = resolveResumePositionSeconds(effectiveWatchProgressPercent, runtimeDurationSeconds);
    const promptOnThisLoad = shouldPromptResumeRef.current;
    const activeResumeSelection = resumeSelection?.loadKey === loadKey ? resumeSelection : null;

    if (
      promptOnThisLoad &&
      startupTargetTime == null &&
      livePlaybackPositionSeconds == null &&
      watchProgressStartTime != null &&
      activeResumeSelection == null
    ) {
      shouldPromptResumeRef.current = false;
      setPlayerRuntimeState({
        targetTime: null,
        errorMessage: null,
        buffering: false,
        hasMediaLoaded: false,
        ended: false,
        shouldPromptResume: false,
      });
      openPrompt(watchProgressStartTime, {
        resume: () => {
          setResumeSelection({ loadKey, startTime: watchProgressStartTime });
        },
        startOver: () => {
          setResumeSelection({ loadKey, startTime: null });
        },
      });
      return;
    }

    shouldPromptResumeRef.current = false;

    const controllerStartTime =
      startupTargetTime ?? livePlaybackPositionSeconds ?? activeResumeSelection?.startTime ?? null;
    const shouldAutoplayOnLoad = runtime.autoplay && session.mode !== "SYNCED";

    setPlayerRuntimeState({
      targetTime: controllerStartTime,
      errorMessage: null,
      buffering: true,
      hasMediaLoaded: false,
      ended: false,
      shouldPromptResume: false,
    });

    let active = true;

    void mintPlaybackUrl({
      input: {
        fileId: media.defaultFile.id,
        playerId: session.playerId,
        videoRenditionId: activeVideoRendition.renditionId,
        audioStreamIndex: activeAudioTrack.streamIndex,
        audioRenditionId: activeAudioRendition.renditionId,
      },
    })
      .then((result) => {
        if (!active) return null;
        if (result.error || !result.data?.mintPlaybackUrl.url) {
          throw result.error ?? new Error("Failed to mint playback URL");
        }
        return createHlsPlayer(video, result.data.mintPlaybackUrl.url, {
          initialPositionSeconds: controllerStartTime,
          shouldAutoplay: () => playerRuntimeStore.getState().autoplay && session.mode !== "SYNCED",
          pauseAfterInitialSeek: controllerStartTime != null && !shouldAutoplayOnLoad,
          videoRef,
          onError: (message) => {
            setPlayerRuntimeState({ errorMessage: message });
          },
          onLoadingChange: (loading) => {
            setPlayerRuntimeState({ buffering: loading });
          },
        });
      })
      .then((controller) => {
        if (!active) {
          controller?.destroy();
          return;
        }
        controllerRef.current = controller ?? null;
      })
      .catch((error) => {
        console.error("failed to start playback", error);
        if (!active) return;
        setPlayerRuntimeState({
          errorMessage: "Sorry, this item is unavailable",
          buffering: false,
        });
      });

    return () => {
      active = false;
      controllerRef.current?.destroy();
      controllerRef.current = null;
    };
  }, [
    activeAudioRendition?.renditionId,
    activeAudioTrack?.streamIndex,
    activeVideoRendition?.renditionId,
    defaultFileId,
    media?.id,
    runtimeMinutes,
    loadKey,
    mintPlaybackUrl,
    openPrompt,
    resumeSelection,
    session.mode,
    session.playerId,
  ]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const resolvePendingTargetTime = () => {
      const pendingTargetTime = playerRuntimeStore.getState().targetTime;
      if (typeof pendingTargetTime !== "number" || !Number.isFinite(pendingTargetTime) || pendingTargetTime < 0) {
        return null;
      }
      return pendingTargetTime;
    };

    const isPendingTargetAhead = (positionSeconds: number) => {
      const pendingTargetTime = resolvePendingTargetTime();
      return pendingTargetTime != null && positionSeconds + 1 < pendingTargetTime;
    };

    const applyPendingTargetTime = () => {
      const pendingTargetTime = resolvePendingTargetTime();
      if (pendingTargetTime == null) return;
      if (Math.abs(video.currentTime - pendingTargetTime) <= 1) return;
      video.currentTime = pendingTargetTime;
    };

    const clearResolvedTargetTime = () => {
      const pendingTargetTime = resolvePendingTargetTime();
      if (pendingTargetTime == null) return;
      const currentTime = Number.isFinite(video.currentTime) ? video.currentTime : 0;
      if (currentTime + 1 < pendingTargetTime) return;
      setPlayerRuntimeState({ targetTime: null });
    };

    const syncSnapshot = (force = false) => {
      if (!media) return;
      if (playerRuntimeStore.getState().currentItemId !== media.id) return;

      const position = Number.isFinite(video.currentTime) && video.currentTime > 0 ? video.currentTime : 0;
      if (isPendingTargetAhead(position)) return;
      const now = Date.now();
      const previous = snapshotUpdateRef.current;
      if (!force) {
        const positionDelta =
          previous.lastPosition == null ? Number.POSITIVE_INFINITY : Math.abs(position - previous.lastPosition);
        if (positionDelta < 1 && now - previous.lastUpdatedAt < 1_000) return;
      }

      playerOptionsStore.getState().setSnapshot({
        currentItemId: media.id,
        position,
      });
      snapshotUpdateRef.current = {
        mediaId: media.id,
        lastPosition: position,
        lastUpdatedAt: now,
      };
    };

    const syncWatchProgress = () => {
      if (!media?.defaultFile || video.duration <= 0) return;
      const currentTime = Number.isFinite(video.currentTime) ? video.currentTime : 0;
      if (isPendingTargetAhead(currentTime)) return;
      const progressPercent = currentTime / video.duration;
      if (!Number.isFinite(progressPercent)) return;

      const previous = watchProgressRef.current;
      if (previous.mediaId !== media.id || previous.fileId !== media.defaultFile.id) {
        watchProgressRef.current = {
          mediaId: media.id,
          fileId: media.defaultFile.id,
          lastProgressPercent: null,
        };
      }
      if (watchProgressRef.current.lastProgressPercent === progressPercent) return;
      watchProgressRef.current.lastProgressPercent = progressPercent;

      void updateWatchProgress({
        fileId: media.defaultFile.id,
        progressPercent,
      }).catch((error) => console.error("failed to update watch state", error));
    };

    const syncAspectRatio = () => {
      if (video.videoWidth <= 0 || video.videoHeight <= 0) return;
      setPlayerRuntimeState({ aspectRatio: video.videoWidth / video.videoHeight });
    };

    const updatePlaybackState = () => {
      setPlayerRuntimeState({
        playing: !video.paused,
        currentTime: Number.isFinite(video.currentTime) ? video.currentTime : 0,
        duration: Number.isFinite(video.duration) ? video.duration : 0,
        buffering: false,
        ended: video.ended,
      });
      playerOptionsStore.getState().setVolume(video.volume);
      playerOptionsStore.getState().setMuted(video.muted);
      clearResolvedTargetTime();
      syncSnapshot();
      syncAspectRatio();
    };

    const handleLoadStart = () => {
      const pendingTargetTime = resolvePendingTargetTime();
      setPlayerRuntimeState({
        currentTime: pendingTargetTime ?? 0,
        duration: 0,
        ended: false,
        buffering: true,
        hasMediaLoaded: false,
        errorMessage: null,
      });
    };

    const handleLoadedMetadata = () => {
      applyPendingTargetTime();
      syncAspectRatio();
      setPlayerRuntimeState({
        hasMediaLoaded: true,
      });
      updatePlaybackState();
    };

    const handleWaiting = () => {
      setPlayerRuntimeState({ buffering: true });
    };

    const handleCanPlay = () => {
      applyPendingTargetTime();
      setPlayerRuntimeState({ buffering: false, hasMediaLoaded: true });
      updatePlaybackState();
    };

    const handleEnded = () => {
      setPlayerRuntimeState({ autoplay: false, ended: true, playing: false, buffering: false });
      syncSnapshot(true);
      syncWatchProgress();
    };

    let lastUpdated = 0;
    const handleTimeUpdate = () => {
      const now = Date.now();
      if (now - lastUpdated < 300) return;
      lastUpdated = now;
      updatePlaybackState();
    };

    const handleSeeked = () => {
      updatePlaybackState();
      syncSnapshot(true);
      syncWatchProgress();
      setPlayerRuntimeState({ targetTime: null, ended: false });
    };

    const watchProgressInterval = window.setInterval(syncWatchProgress, 5_000);
    video.addEventListener("timeupdate", handleTimeUpdate);
    video.addEventListener("play", updatePlaybackState);
    video.addEventListener("pause", updatePlaybackState);
    video.addEventListener("volumechange", updatePlaybackState);
    video.addEventListener("loadstart", handleLoadStart);
    video.addEventListener("loadedmetadata", handleLoadedMetadata);
    video.addEventListener("canplay", handleCanPlay);
    video.addEventListener("loadeddata", handleCanPlay);
    video.addEventListener("playing", handleCanPlay);
    video.addEventListener("waiting", handleWaiting);
    video.addEventListener("ended", handleEnded);
    video.addEventListener("seeked", handleSeeked);
    video.addEventListener("resize", syncAspectRatio);

    return () => {
      syncSnapshot(true);
      window.clearInterval(watchProgressInterval);
      video.removeEventListener("timeupdate", handleTimeUpdate);
      video.removeEventListener("play", updatePlaybackState);
      video.removeEventListener("pause", updatePlaybackState);
      video.removeEventListener("volumechange", updatePlaybackState);
      video.removeEventListener("loadstart", handleLoadStart);
      video.removeEventListener("loadedmetadata", handleLoadedMetadata);
      video.removeEventListener("canplay", handleCanPlay);
      video.removeEventListener("loadeddata", handleCanPlay);
      video.removeEventListener("playing", handleCanPlay);
      video.removeEventListener("waiting", handleWaiting);
      video.removeEventListener("ended", handleEnded);
      video.removeEventListener("seeked", handleSeeked);
      video.removeEventListener("resize", syncAspectRatio);
    };
  }, [media?.defaultFile, media?.defaultFile?.id, media?.id, updateWatchProgress]);

  const className = useMemo(
    () =>
      isFullscreen
        ? "block h-full w-full bg-black object-contain outline-none"
        : "block h-full w-full rounded bg-black object-contain outline-none",
    [isFullscreen],
  );

  return (
    <video
      ref={videoRef}
      className={className}
      autoPlay={autoplay && session.mode !== "SYNCED"}
      controls={false}
      disablePictureInPicture
    />
  );
};
