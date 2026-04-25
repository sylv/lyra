import { CaptionsRenderer, parseResponse } from "media-captions";
import { useEffect, useRef, useState, type FC } from "react";
import { useMutation } from "urql";
import { graphql } from "../../../@generated/gql";
import { type ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { cn } from "../../../lib/utils";
import { setPlayerRuntimeState, usePlayerRuntimeStore } from "../player-runtime-store";
import { usePlayerVideoElement } from "../player-video-context";
import { usePlayerVisibility } from "../player-visibility";
import { listPreferredSubtitleRenditions, subtitleRenditionFormat } from "../subtitles";
import "./player-subtitle-overlay.css";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;
type ActiveSubtitle = { key: string; language: string | null };

const MintSubtitleUrl = graphql(`
  mutation MintSubtitleUrl($input: SubtitleUrlInput!) {
    mintSubtitleUrl(input: $input) {
      url
    }
  }
`);

const safeCurrentTime = (videoElement: HTMLVideoElement | null) =>
  videoElement && Number.isFinite(videoElement.currentTime) ? videoElement.currentTime : 0;

const clearSubtitleRuntimeState = () => {
  setPlayerRuntimeState({
    activeSubtitleTrackId: null,
    activeSubtitleRenditionId: null,
    pendingSubtitleTrackId: null,
  });
};

export const PlayerSubtitleOverlay: FC<{ media: CurrentMedia | null }> = ({ media }) => {
  const { showControls } = usePlayerVisibility();
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);
  const selectedSubtitleTrackId = usePlayerRuntimeStore((state) => state.selectedSubtitleTrackId);
  const videoElement = usePlayerVideoElement();
  const overlayRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<CaptionsRenderer | null>(null);
  const subtitleUrlCacheRef = useRef(new Map<string, string>());
  const subtitleRequestIdRef = useRef(0);
  const [activeSubtitle, setActiveSubtitle] = useState<ActiveSubtitle | null>(null);
  const [, mintSubtitleUrl] = useMutation(MintSubtitleUrl);
  const playbackOptions = media?.defaultFile?.playbackOptions ?? null;
  const subtitleTracks = playbackOptions?.subtitleTracks ?? [];
  const autoselectSubtitleTrack = subtitleTracks.find((track) => track.autoselect) ?? null;
  const defaultFileId = media?.defaultFile?.id ?? null;

  const syncRendererTime = (timeSeconds: number) => {
    if (rendererRef.current) {
      rendererRef.current.currentTime = Number.isFinite(timeSeconds) ? Math.max(0, timeSeconds) : 0;
    }
  };

  useEffect(() => {
    const overlay = overlayRef.current;
    if (!overlay) return;

    const renderer = new CaptionsRenderer(overlay);
    rendererRef.current = renderer;
    return () => {
      renderer.destroy();
      if (rendererRef.current === renderer) {
        rendererRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;

    const fileId = defaultFileId;
    const resolvedTrack =
      selectedSubtitleTrackId === ""
        ? null
        : selectedSubtitleTrackId == null
          ? autoselectSubtitleTrack
          : (subtitleTracks.find((track) => track.id === selectedSubtitleTrackId) ?? null);
    const clearActiveSubtitle = () => {
      setActiveSubtitle(null);
      renderer.reset();
      clearSubtitleRuntimeState();
    };

    if (!fileId || !resolvedTrack) {
      subtitleRequestIdRef.current += 1;
      clearActiveSubtitle();
      return;
    }

    const renditions = listPreferredSubtitleRenditions(resolvedTrack);
    if (renditions.length === 0) {
      subtitleRequestIdRef.current += 1;
      clearActiveSubtitle();
      return;
    }

    let cancelled = false;
    const controller = new AbortController();
    const requestId = subtitleRequestIdRef.current + 1;
    subtitleRequestIdRef.current = requestId;
    const isRequestStale = () => cancelled || controller.signal.aborted || subtitleRequestIdRef.current !== requestId;

    renderer.reset();
    setActiveSubtitle(null);
    setPlayerRuntimeState({
      activeSubtitleTrackId: null,
      activeSubtitleRenditionId: null,
      pendingSubtitleTrackId: resolvedTrack.id,
    });

    void (async () => {
      for (const rendition of renditions) {
        const format = subtitleRenditionFormat(rendition);
        if (!format) continue;

        const cacheKey = `${resolvedTrack.id}:${rendition.id}`;
        let url = subtitleUrlCacheRef.current.get(cacheKey) ?? null;
        if (!url) {
          const result = await mintSubtitleUrl({
            input: {
              fileId,
              trackId: resolvedTrack.id,
              renditionId: rendition.id,
              manual: selectedSubtitleTrackId != null && selectedSubtitleTrackId !== "",
            },
          });
          if (result.error || !result.data?.mintSubtitleUrl.url) {
            console.warn("failed to mint preferred subtitle rendition", {
              trackId: resolvedTrack.id,
              renditionId: rendition.id,
              error: result.error,
            });
            continue;
          }
          url = result.data.mintSubtitleUrl.url;
          subtitleUrlCacheRef.current.set(cacheKey, url);
        }

        if (isRequestStale() || !url) return;

        try {
          const track = await parseResponse(fetch(url, { signal: controller.signal }), { type: format });
          if (isRequestStale()) return;

          renderer.changeTrack(track);
          syncRendererTime(safeCurrentTime(videoElement));
          setActiveSubtitle({
            key: cacheKey,
            language: resolvedTrack.languageBcp47 ?? null,
          });
          setPlayerRuntimeState({
            activeSubtitleTrackId: resolvedTrack.id,
            activeSubtitleRenditionId: rendition.id,
            pendingSubtitleTrackId: null,
          });
          return;
        } catch (error) {
          if (isRequestStale()) return;
          subtitleUrlCacheRef.current.delete(cacheKey);
          console.warn("failed to load subtitle rendition", {
            trackId: resolvedTrack.id,
            renditionId: rendition.id,
            error,
          });
        }
      }

      clearActiveSubtitle();
      console.error("failed to apply subtitle track");
    })().catch((error) => {
      if (controller.signal.aborted) return;
      clearActiveSubtitle();
      console.error("failed to load subtitle track", error);
    });

    return () => {
      cancelled = true;
      controller.abort();
      setActiveSubtitle(null);
      renderer.reset();
    };
  }, [autoselectSubtitleTrack, defaultFileId, mintSubtitleUrl, selectedSubtitleTrackId, subtitleTracks]);

  // Keep captions on the same timeline exposed by the media element. On HLS/MSE sources,
  // frame metadata can drift from `currentTime` and shift cues by a few seconds.
  useEffect(() => {
    const video = videoElement;
    if (!video || !activeSubtitle) return;

    let frameId: number | null = null;
    const syncNow = () => syncRendererTime(safeCurrentTime(video));

    const stopSync = () => {
      if (frameId == null) return;
      window.cancelAnimationFrame(frameId);
      frameId = null;
    };

    const tick = () => {
      syncNow();
      frameId = window.requestAnimationFrame(tick);
    };

    const startSync = () => {
      if (frameId != null) return;
      frameId = window.requestAnimationFrame(tick);
    };

    const handlePause = () => {
      stopSync();
      syncNow();
    };

    syncNow();
    if (!video.paused && !video.ended) {
      startSync();
    }

    video.addEventListener("play", startSync);
    video.addEventListener("playing", startSync);
    video.addEventListener("pause", handlePause);
    video.addEventListener("waiting", handlePause);
    video.addEventListener("timeupdate", syncNow);
    video.addEventListener("seeking", syncNow);
    video.addEventListener("seeked", syncNow);
    video.addEventListener("ratechange", syncNow);
    video.addEventListener("loadedmetadata", syncNow);
    video.addEventListener("ended", handlePause);

    return () => {
      stopSync();
      video.removeEventListener("play", startSync);
      video.removeEventListener("playing", startSync);
      video.removeEventListener("pause", handlePause);
      video.removeEventListener("waiting", handlePause);
      video.removeEventListener("timeupdate", syncNow);
      video.removeEventListener("seeking", syncNow);
      video.removeEventListener("seeked", syncNow);
      video.removeEventListener("ratechange", syncNow);
      video.removeEventListener("loadedmetadata", syncNow);
      video.removeEventListener("ended", handlePause);
    };
  }, [activeSubtitle?.key, videoElement]);

  return (
    <div className={cn("pointer-events-none absolute inset-x-0 bottom-2 top-2 z-20")} aria-live="polite">
      <div
        ref={overlayRef}
        data-part="captions"
        lang={activeSubtitle?.language ?? undefined}
        data-fullscreen={isFullscreen}
        className={cn(
          "player-subtitle-overlay absolute inset-0 transition-transform duration-300 ease-out will-change-transform",
          showControls ? "-translate-y-20" : "translate-y-0",
        )}
      />
    </div>
  );
};
