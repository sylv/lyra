import { CaptionsRenderer, parseResponse, type CaptionsFileFormat } from "media-captions";
import { useEffect, useMemo, useRef, useState, type FC } from "react";
import { PlaybackSubtitleCodec } from "../../../@generated/gql/graphql";
import { cn } from "../../../lib/utils";
import { PlayerDynamicMiddle } from "../ui/player-dynamic-middle";
import { PlayerState, usePlayerStore } from "../store/player-store";
import "./player-subtitle-overlay.css";

const subtitleFormatRank = (format: CaptionsFileFormat) => {
  switch (format) {
    case "vtt":
      return 0;
    case "srt":
      return 1;
    case "ass":
    case "ssa":
      return 2;
  }
};

const subtitleRenditionFormat = (codec: PlaybackSubtitleCodec): CaptionsFileFormat | null => {
  switch (codec) {
    case PlaybackSubtitleCodec.Vtt:
      return "vtt";
    case PlaybackSubtitleCodec.Srt:
      return "srt";
    case PlaybackSubtitleCodec.Ass:
      return "ass";
    default:
      return null;
  }
};

const safeCurrentTime = (videoElement: HTMLVideoElement | null) =>
  videoElement && Number.isFinite(videoElement.currentTime) ? videoElement.currentTime : 0;

export const PlayerSubtitleOverlay: FC = () => {
  const status = usePlayerStore((state) => state.status);
  const isFullscreen = usePlayerStore((state) => state.isFullscreen);
  const selectedSubtitleTrackId = usePlayerStore((state) => state.selectedSubtitleTrackId);
  const videoRef = usePlayerStore((state) => state.videoRef);
  const overlayRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<CaptionsRenderer | null>(null);
  const requestIdRef = useRef(0);
  const [activeLanguage, setActiveLanguage] = useState<string | null>(null);

  const subtitleTracks = status.state === PlayerState.Mounted ? status.subtitleTracks : [];
  const autoselectTrack = useMemo(() => subtitleTracks.find((track) => track.autoselect) ?? null, [subtitleTracks]);

  useEffect(() => {
    const overlay = overlayRef.current;
    if (!overlay) return;

    const renderer = new CaptionsRenderer(overlay);
    rendererRef.current = renderer;
    return () => {
      renderer.destroy();
      if (rendererRef.current === renderer) rendererRef.current = null;
    };
  }, []);

  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;

    const resolvedTrack =
      selectedSubtitleTrackId === ""
        ? null
        : selectedSubtitleTrackId == null
          ? autoselectTrack
          : (subtitleTracks.find((track) => track.sourceTrackId === selectedSubtitleTrackId) ?? null);

    const clear = () => {
      renderer.reset();
      setActiveLanguage(null);
      usePlayerStore.setState({
        activeSubtitleTrackId: null,
        activeSubtitleRenditionId: null,
        pendingSubtitleTrackId: null,
      });
    };

    if (!resolvedTrack) {
      requestIdRef.current += 1;
      clear();
      return;
    }

    const preferredRenditions = [...resolvedTrack.renditions]
      .map((rendition) => ({ rendition, format: subtitleRenditionFormat(rendition.codec) }))
      .filter((entry): entry is typeof entry & { format: CaptionsFileFormat } => entry.format != null)
      .sort((left, right) => subtitleFormatRank(left.format) - subtitleFormatRank(right.format));

    if (preferredRenditions.length === 0) {
      requestIdRef.current += 1;
      clear();
      return;
    }

    let cancelled = false;
    const controller = new AbortController();
    const requestId = requestIdRef.current + 1;
    requestIdRef.current = requestId;
    const isStale = () => cancelled || controller.signal.aborted || requestIdRef.current !== requestId;

    renderer.reset();
    setActiveLanguage(null);
    usePlayerStore.setState({
      activeSubtitleTrackId: null,
      activeSubtitleRenditionId: null,
      pendingSubtitleTrackId: resolvedTrack.sourceTrackId,
    });

    void (async () => {
      for (const { rendition, format } of preferredRenditions) {
        try {
          const captions = await parseResponse(fetch(rendition.signedUrl, { signal: controller.signal }), {
            type: format,
          });
          if (isStale()) return;

          renderer.changeTrack(captions);
          renderer.currentTime = safeCurrentTime(videoRef.current);
          setActiveLanguage(resolvedTrack.languageBcp47 ?? null);
          usePlayerStore.setState({
            activeSubtitleTrackId: resolvedTrack.sourceTrackId,
            activeSubtitleRenditionId: rendition.variantId ?? rendition.displayInfo,
            pendingSubtitleTrackId: null,
          });
          return;
        } catch (error) {
          if (isStale()) return;
          console.warn("failed to load subtitle rendition", { track: resolvedTrack.sourceTrackId, rendition, error });
        }
      }

      clear();
    })().catch((error) => {
      if (controller.signal.aborted) return;
      clear();
      console.error("failed to load subtitle track", error);
    });

    return () => {
      cancelled = true;
      controller.abort();
      setActiveLanguage(null);
      renderer.reset();
    };
  }, [autoselectTrack, selectedSubtitleTrackId, subtitleTracks, videoRef]);

  useEffect(() => {
    const video = videoRef.current;
    const renderer = rendererRef.current;
    if (!video || !renderer || !activeLanguage) return;

    let frameId: number | null = null;
    const syncNow = () => {
      renderer.currentTime = safeCurrentTime(video);
    };
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
    if (!video.paused && !video.ended) startSync();

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
  }, [activeLanguage, videoRef]);

  return (
    <PlayerDynamicMiddle>
      <div
        ref={overlayRef}
        data-part="captions"
        lang={activeLanguage ?? undefined}
        data-fullscreen={isFullscreen}
        className={cn("player-subtitle-overlay absolute inset-0 transition-transform duration-300 ease-out")}
      />
    </PlayerDynamicMiddle>
  );
};
