import type { RefObject } from "react";

export interface ResumeConfig {
  initialPositionSeconds: number | null;
  shouldAutoplay: () => boolean;
  pauseAfterInitialSeek: boolean;
  videoRef: RefObject<HTMLVideoElement | null>;
  onError: (message: string) => void;
  onLoadingChange: (loading: boolean) => void;
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

  const { initialPositionSeconds, shouldAutoplay, pauseAfterInitialSeek, videoRef, onError, onLoadingChange } =
    resumeConfig;

  let hasStartedLoading = false;
  const startLoadAt = (startPosition: number) => {
    if (hasStartedLoading) return;
    hasStartedLoading = true;
    const autoplayRequested = shouldAutoplay();
    if (videoRef.current) {
      videoRef.current.autoplay = autoplayRequested;
    }
    hls.startLoad(Number.isFinite(startPosition) ? startPosition : -1, true);
    if (autoplayRequested && videoRef.current) {
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
      if (videoRef.current && pauseAfterInitialSeek && !shouldAutoplay()) {
        videoRef.current.pause();
      }
      startLoadAt(initialPositionSeconds);
      return;
    }

    startLoadAt(-1);
  });

  hls.loadSource(hlsUrl);
  hls.attachMedia(video);

  return {
    destroy() {
      hls.destroy();
    },
  };
};
