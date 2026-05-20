// oxlint-disable jsx_a11y/media-has-caption
import { Fragment, useCallback, useEffect, useMemo, useRef, type FC, type ReactNode } from "react";
import { PlayerState, setPlayerError, togglePlayerMute, usePlayerStore } from "./store/player-store";
import Hls from "hls.js";
import { PlayerVideoContext } from "./store/player-video-context";

const MINIPLAYER_HEIGHT = 360;
const HLS_TIMEOUT_MS = 15_000;
const HLS_RETRY_DELAY_MS = 1000;
const HLS_MAX_RETRY_TIME = 300_000;
const HLS_RETRY_COUNT = Math.ceil(HLS_MAX_RETRY_TIME / HLS_TIMEOUT_MS);
const SURFACE_PAUSE_COMMIT_MS = 180;
const SURFACE_DOUBLE_CLICK_WINDOW_MS = 460;

const retryPolicy = {
  maxNumRetry: HLS_RETRY_COUNT,
  retryDelayMs: HLS_RETRY_DELAY_MS,
  maxRetryDelayMs: HLS_RETRY_DELAY_MS,
  backoff: "linear" as const,
};

const loaderPolicy = {
  default: {
    maxTimeToFirstByteMs: HLS_TIMEOUT_MS,
    maxLoadTimeMs: HLS_TIMEOUT_MS,
    timeoutRetry: retryPolicy,
    errorRetry: retryPolicy,
  },
};

const isBufferedAt = (video: HTMLVideoElement, time: number) => {
  for (let i = 0; i < video.buffered.length; i++) {
    if (time >= video.buffered.start(i) && time <= video.buffered.end(i)) return true;
  }
  return false;
};

export const PlayerVideo: FC<{ children: ReactNode }> = ({ children }) => {
  const status = usePlayerStore((state) => state.status);
  const currentTime = usePlayerStore((state) => state.currentTime);
  const aspectRatio = usePlayerStore((state) => state.aspectRatio);
  const isFullscreen = usePlayerStore((state) => state.isFullscreen);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const hlsRef = useRef<Hls | null>(null);
  const surfaceClickRef = useRef<{
    wasPaused: boolean;
    pauseTimer: number | null;
    singleClickTimer: number | null;
  } | null>(null);
  const volume = usePlayerStore((state) => state.volume);
  const muted = usePlayerStore((state) => state.muted);
  const playbackRate = usePlayerStore((state) => state.playbackRate);
  const selectedVideoRenditionPairId = usePlayerStore((state) => state.selectedVideoRenditionPairId);
  const selectedAudioTrackId = usePlayerStore((state) => state.selectedAudioTrackId);

  useEffect(() => {
    usePlayerStore.setState({ videoRef: videoRef });
  }, [videoRef]);

  const hlsSelection = useMemo(() => {
    if (status.state !== PlayerState.Mounted || !videoRef.current) {
      return { hlsUrl: null, videoRenditionOptions: [], audioTrackOptions: [] };
    }
    let videoRendition = null;
    let audioRendition = null;
    const selectedAudioTrack = selectedAudioTrackId
      ? (status.audioTracks.find((track) => track.sourceTrackId === selectedAudioTrackId) ?? status.audioTrack)
      : status.audioTrack;

    const videoRenditionOptions = status.videoTracks.flatMap((track) =>
      track.renditions.map((rendition) => ({
        track,
        rendition,
        compatibility: videoRef.current!.canPlayType(`video/mp4; codecs="${rendition.codecTag}"`),
      })),
    );
    for (const { rendition, compatibility } of videoRenditionOptions) {
      if (
        (selectedVideoRenditionPairId === rendition.pairId || !selectedVideoRenditionPairId) &&
        !videoRendition &&
        (compatibility === "probably" || compatibility === "maybe")
      ) {
        videoRendition = rendition;
      }
    }

    const audioTrackOptions = status.audioTracks.map((track) => ({
      track,
      supportedRenditions: track.renditions
        .map((rendition) => ({
          rendition,
          compatibility: videoRef.current!.canPlayType(`audio/mp4; codecs="${rendition.codecTag}"`),
        }))
        .filter(({ compatibility }) => compatibility === "probably" || compatibility === "maybe"),
    }));
    if (selectedAudioTrack) {
      const selectedAudioOption = audioTrackOptions.find(
        (option) => option.track.sourceTrackId === selectedAudioTrack.sourceTrackId,
      );
      for (const { rendition } of selectedAudioOption?.supportedRenditions ?? []) {
        if (!audioRendition) {
          audioRendition = rendition;
        }
      }
    }

    if (!videoRendition) {
      setPlayerError("All available video renditions are unsupported");
      return { hlsUrl: null, videoRenditionOptions, audioTrackOptions };
    }

    const templateUrl = status.data.node.defaultFile?.playback.hlsUrlTemplate;
    if (!templateUrl) throw new Error("HLS URL template is not available");

    let finalUrl = templateUrl.replace("{VIDEO_PAIR_ID}", videoRendition.pairId);
    if (audioRendition) {
      finalUrl = finalUrl.replace("{AUDIO_PAIR_ID}", audioRendition.pairId);
    } else {
      finalUrl = finalUrl.replace("{AUDIO_PAIR_ID}", "none");
    }

    return { hlsUrl: finalUrl, videoRenditionOptions, audioTrackOptions };
  }, [status, videoRef, selectedAudioTrackId, selectedVideoRenditionPairId]);

  const hlsUrl = hlsSelection.hlsUrl;

  useEffect(() => {
    usePlayerStore.setState({
      videoRenditionOptions: hlsSelection.videoRenditionOptions,
      audioTrackOptions: hlsSelection.audioTrackOptions,
    });
  }, [hlsSelection.audioTrackOptions, hlsSelection.videoRenditionOptions]);

  useEffect(() => {
    if (!hlsUrl || !videoRef.current) return;
    if (!Hls.isSupported()) {
      // todo: technically we should fall back to checking for native hls support,
      // but native hls support is usually too strict to support our hls playlists (for the time being anyway)
      setPlayerError("Your browser is incompatible with the video player.");
      return;
    }

    if (hlsRef.current) {
      hlsRef.current.destroy();
    }

    // todo: hack
    const initialSeekAfterBuffer = Number.isFinite(currentTime) && currentTime > 0 ? currentTime : null;
    let hasAppliedInitialSeek = initialSeekAfterBuffer == null;
    const hls = new Hls({
      autoStartLoad: false,
      // todo: hack
      startPosition: 0,
      manifestLoadPolicy: loaderPolicy,
      playlistLoadPolicy: loaderPolicy,
      fragLoadPolicy: loaderPolicy,
    });

    hlsRef.current = hls;
    hls.loadSource(hlsUrl);
    hls.attachMedia(videoRef.current);
    hls.on(Hls.Events.ERROR, (event, data) => {
      console.error("HLS error:", event, data);
      if (data.fatal) {
        setPlayerError(`${data.type}: ${data.reason}`);
      }
    });
    hls.on(Hls.Events.MANIFEST_PARSED, () => {
      if (!videoRef.current) return;
      // todo: hack
      hls.startLoad(0);
      void videoRef.current.play().catch(() => undefined);
    });
    // once the video loads, update the duration in the store
    hls.on(Hls.Events.FRAG_LOADED, () => {
      if (!videoRef.current) return;
      const { aspectRatio, durationSeconds } = usePlayerStore.getState();
      const nextAspectRatio = videoRef.current.videoWidth / videoRef.current.videoHeight;
      if (videoRef.current.duration === durationSeconds && aspectRatio === nextAspectRatio) return;
      usePlayerStore.setState({
        durationSeconds: videoRef.current.duration,
        aspectRatio: nextAspectRatio,
      });
    });

    const syncBufferedRanges = () => {
      if (!videoRef.current) return;
      const ranges: Array<{ start: number; end: number }> = [];
      for (let i = 0; i < videoRef.current.buffered.length; i++) {
        ranges.push({ start: videoRef.current.buffered.start(i), end: videoRef.current.buffered.end(i) });
      }
      usePlayerStore.setState({ bufferedRanges: ranges });
    };

    hls.on(Hls.Events.BUFFER_APPENDED, () => {
      if (videoRef.current) {
        // todo: hack
        if (!hasAppliedInitialSeek && initialSeekAfterBuffer != null) {
          videoRef.current.currentTime = initialSeekAfterBuffer;
          hasAppliedInitialSeek = true;
        }
        usePlayerStore.setState({
          buffering: videoRef.current.readyState < 3 && !isBufferedAt(videoRef.current, videoRef.current.currentTime),
        });
      }
      syncBufferedRanges();
    });

    hls.on(Hls.Events.BUFFER_FLUSHED, () => {
      syncBufferedRanges();
    });
    return () => {
      hls.destroy();
      if (hlsRef.current === hls) hlsRef.current = null;
    };
  }, [hlsUrl, videoRef]);

  useEffect(() => {
    const videoEl = videoRef.current;
    if (!videoEl) return;
    videoEl.volume = volume;
    videoEl.muted = muted;
    videoEl.playbackRate = playbackRate;
  }, [muted, playbackRate, volume]);

  useEffect(() => {
    // sync video state to store
    const videoEl = videoRef.current;
    if (!videoEl) return;

    usePlayerStore.setState({
      buffering: videoEl.readyState < 3,
      paused: videoEl.paused,
    });

    const syncBufferedRanges = () => {
      const ranges: Array<{ start: number; end: number }> = [];
      for (let i = 0; i < videoEl.buffered.length; i++) {
        ranges.push({ start: videoEl.buffered.start(i), end: videoEl.buffered.end(i) });
      }
      usePlayerStore.setState({ bufferedRanges: ranges });
    };
    const syncBuffering = () => {
      syncBufferedRanges();
      usePlayerStore.setState({
        buffering:
          !videoEl.paused && !videoEl.ended && videoEl.readyState < 3 && !isBufferedAt(videoEl, videoEl.currentTime),
      });
    };
    const onTimeUpdate = () => {
      usePlayerStore.setState({ currentTime: videoEl.currentTime });
      syncBuffering();
    };
    const onPlay = () => usePlayerStore.setState({ paused: false, ended: false });
    const onPause = () => usePlayerStore.setState({ paused: true });
    const onEnded = () => usePlayerStore.setState({ paused: true, ended: true, buffering: false });
    const onBuffering = () => syncBuffering();
    const onReady = () => usePlayerStore.setState({ buffering: false });

    videoEl.addEventListener("timeupdate", onTimeUpdate);
    videoEl.addEventListener("play", onPlay);
    videoEl.addEventListener("pause", onPause);
    videoEl.addEventListener("waiting", onBuffering);
    videoEl.addEventListener("seeking", onBuffering);
    videoEl.addEventListener("canplay", onReady);
    videoEl.addEventListener("playing", onReady);
    videoEl.addEventListener("progress", syncBufferedRanges);
    videoEl.addEventListener("ended", onEnded);

    return () => {
      videoEl.removeEventListener("timeupdate", onTimeUpdate);
      videoEl.removeEventListener("play", onPlay);
      videoEl.removeEventListener("pause", onPause);
      videoEl.removeEventListener("waiting", onBuffering);
      videoEl.removeEventListener("seeking", onBuffering);
      videoEl.removeEventListener("canplay", onReady);
      videoEl.removeEventListener("playing", onReady);
      videoEl.removeEventListener("progress", syncBufferedRanges);
      videoEl.removeEventListener("ended", onEnded);
    };
  }, [videoRef]);

  const clearSurfaceClick = useCallback(() => {
    const pending = surfaceClickRef.current;
    if (!pending) return;
    if (pending.pauseTimer != null) window.clearTimeout(pending.pauseTimer);
    if (pending.singleClickTimer != null) window.clearTimeout(pending.singleClickTimer);
    surfaceClickRef.current = null;
  }, []);

  useEffect(() => clearSurfaceClick, [clearSurfaceClick]);

  const playVideo = useCallback(() => {
    const videoEl = videoRef.current;
    if (!videoEl) return;
    usePlayerStore.setState({ paused: false, ended: false });
    void videoEl.play().catch(() => undefined);
  }, []);

  const pauseVideo = useCallback(() => {
    const videoEl = videoRef.current;
    if (!videoEl) return;
    videoEl.pause();
    usePlayerStore.setState({ paused: true });
  }, []);

  const togglePlaying = useCallback(() => {
    const videoEl = videoRef.current;
    if (!videoEl) return;
    const wasPaused = usePlayerStore.getState().paused;
    clearSurfaceClick();
    if (wasPaused) {
      playVideo();
    } else {
      pauseVideo();
    }
  }, [clearSurfaceClick, pauseVideo, playVideo]);

  const toggleSurfacePlaying = useCallback(() => {
    const videoEl = videoRef.current;
    if (!videoEl) return;

    const pending = surfaceClickRef.current;
    if (pending) {
      const wasPaused = pending.wasPaused;
      clearSurfaceClick();
      if (!wasPaused) playVideo();
      usePlayerStore.setState((state) => {
        state.isFullscreen = !state.isFullscreen;
      });
      return;
    }

    const wasPaused = usePlayerStore.getState().paused;
    if (wasPaused) {
      surfaceClickRef.current = {
        wasPaused,
        pauseTimer: null,
        singleClickTimer: window.setTimeout(() => {
          surfaceClickRef.current = null;
          playVideo();
        }, SURFACE_DOUBLE_CLICK_WINDOW_MS),
      };
      return;
    }

    // Match YouTube's feel: the chrome flips to paused immediately, but playback
    // keeps moving briefly so a double click can become fullscreen without a hitch.
    usePlayerStore.setState({ paused: true });
    surfaceClickRef.current = {
      wasPaused,
      pauseTimer: window.setTimeout(() => {
        if (!surfaceClickRef.current) return;
        pauseVideo();
      }, SURFACE_PAUSE_COMMIT_MS),
      singleClickTimer: window.setTimeout(() => {
        surfaceClickRef.current = null;
      }, SURFACE_DOUBLE_CLICK_WINDOW_MS),
    };
  }, [clearSurfaceClick, pauseVideo, playVideo]);

  const seek = useCallback(
    (time: number) => {
      const videoEl = videoRef.current;
      if (!videoEl) return;
      clearSurfaceClick();
      videoEl.currentTime = time;
    },
    [clearSurfaceClick],
  );

  const toggleMute = useCallback(() => {
    clearSurfaceClick();
    togglePlayerMute();
  }, [clearSurfaceClick]);

  return (
    <Fragment>
      <video
        ref={videoRef}
        controls={false}
        className="bg-black"
        style={
          isFullscreen
            ? { aspectRatio, height: "100dvh", width: "100dvw" }
            : {
                maxHeight: MINIPLAYER_HEIGHT,
                height: MINIPLAYER_HEIGHT,
                aspectRatio: aspectRatio,
              }
        }
      />
      <PlayerVideoContext.Provider value={{ togglePlaying, toggleSurfacePlaying, seek, toggleMute }}>
        {children}
      </PlayerVideoContext.Provider>
    </Fragment>
  );
};
