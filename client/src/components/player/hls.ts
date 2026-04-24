import type { RefObject } from "react";

export interface ResumeConfig {
  initialPositionSeconds: number | null;
  watchProgressPercent: number | null | undefined;
  runtimeDurationSeconds: number | null;
  shouldPromptResume: boolean;
  shouldAutoplay: boolean;
  pauseAfterInitialSeek: boolean;
  videoRef: RefObject<HTMLVideoElement | null>;
  onError: (message: string) => void;
  onLoadingChange: (loading: boolean) => void;
  onResumePrompt: (positionSeconds: number, handlers: { resume: () => void; startOver: () => void }) => void;
}

export interface PlayerController {
  destroy(): void;
}

const HLS_TIMEOUT_MS = 15_000;
const HLS_RETRY_DELAY_MS = 1000;
const HLS_MAX_RETRY_TIME = 300_000;
const HLS_RETRY_COUNT = Math.ceil(HLS_MAX_RETRY_TIME / HLS_TIMEOUT_MS);

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

export const createHlsPlayer = async (
  video: HTMLVideoElement,
  hlsUrl: string,
  resumeConfig: ResumeConfig,
): Promise<PlayerController | null> => {
  const { default: Hls } = await import("hls.js");

  if (!Hls.isSupported()) {
    resumeConfig.onError("Sorry, your browser does not support this video.");
    resumeConfig.onLoadingChange(false);
    return null;
  }

  const {
    initialPositionSeconds,
    watchProgressPercent,
    runtimeDurationSeconds,
    shouldPromptResume,
    shouldAutoplay,
    pauseAfterInitialSeek,
    videoRef,
    onError,
    onLoadingChange,
    onResumePrompt,
  } = resumeConfig;

  const hasResumableWatchProgress =
    typeof watchProgressPercent === "number" &&
    Number.isFinite(watchProgressPercent) &&
    watchProgressPercent > 0 &&
    watchProgressPercent < 1;
  const safeWatchProgressPercent = hasResumableWatchProgress ? watchProgressPercent : 0;

  const clampResumePosition = (durationSeconds: number) => {
    if (!hasResumableWatchProgress) return null;
    const progress = Math.max(0, Math.min(0.999, safeWatchProgressPercent));
    const maxStart = Math.max(0, durationSeconds - 0.5);
    return Math.max(0, Math.min(progress * durationSeconds, maxStart));
  };

  let hasStartedLoading = false;
  const startLoadAt = (startPosition: number) => {
    if (hasStartedLoading) return;
    hasStartedLoading = true;
    if (videoRef.current) {
      videoRef.current.autoplay = shouldAutoplay;
    }
    hls.startLoad(Number.isFinite(startPosition) ? startPosition : -1);
    if (shouldAutoplay && videoRef.current) {
      void videoRef.current.play().catch(() => undefined);
    }
  };

  const hls = new Hls({
    autoStartLoad: false,
    manifestLoadPolicy: loaderPolicy,
    playlistLoadPolicy: loaderPolicy,
    fragLoadPolicy: loaderPolicy,
  });

  hls.on(Hls.Events.ERROR, (event, data) => {
    console.error("HLS error:", event, data);
    if (data.fatal) {
      onError(`${data.type}: ${data.reason}`);
      onLoadingChange(false);
    }
  });

  hls.on(Hls.Events.MANIFEST_PARSED, () => {
    if (
      typeof initialPositionSeconds === "number" &&
      Number.isFinite(initialPositionSeconds) &&
      initialPositionSeconds >= 0
    ) {
      if (videoRef.current) {
        videoRef.current.currentTime = initialPositionSeconds;
        if (pauseAfterInitialSeek) videoRef.current.pause();
      }
      startLoadAt(initialPositionSeconds);
      return;
    }

    if (!hasResumableWatchProgress) {
      startLoadAt(-1);
      return;
    }

    const durationSeconds = hls.levels[0]?.details?.totalduration ?? runtimeDurationSeconds;
    const resumePosition = durationSeconds == null ? null : clampResumePosition(durationSeconds);

    if (resumePosition == null) {
      startLoadAt(-1);
      return;
    }

    if (shouldPromptResume) {
      if (videoRef.current) {
        videoRef.current.autoplay = false;
        videoRef.current.pause();
      }
      onResumePrompt(resumePosition, {
        resume: () => {
          if (videoRef.current) {
            videoRef.current.currentTime = resumePosition;
          }
          startLoadAt(resumePosition);
        },
        startOver: () => {
          startLoadAt(-1);
        },
      });
      return;
    }

    if (videoRef.current) {
      videoRef.current.currentTime = resumePosition;
    }
    startLoadAt(resumePosition);
  });

  hls.loadSource(hlsUrl);
  hls.attachMedia(video);

  return {
    destroy() {
      hls.destroy();
    },
  };
};
