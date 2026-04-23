import { useEffect, useState, type CSSProperties, type FC } from "react";
import { cn } from "../../../lib/utils";
import { usePlayerContext } from "../player-context";
import { usePlayerVideoElement } from "../player-refs-context";

type SubtitleCue = TextTrackCue & {
  align?: string;
  line?: number | "auto";
  position?: number | "auto";
  size?: number;
  snapToLines?: boolean;
};

const escapeCueText = (text: string) =>
  text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");

const cueHtmlToString = (cue: TextTrackCue) => {
  if (!("getCueAsHTML" in cue) || typeof cue.getCueAsHTML !== "function") {
    return "";
  }

  const cueHtml = cue.getCueAsHTML();
  const container = document.createElement("div");
  container.append(cueHtml.cloneNode(true));
  return container.innerHTML;
};

const cueText = (cue: TextTrackCue) => (cue instanceof VTTCue ? cue.text : "");

const cueKey = (cue: TextTrackCue, index: number) => cue.id || `${cue.startTime}:${cue.endTime}:${index}`;

const cueAlignClassName = (cue: SubtitleCue) => {
  switch (cue.align ?? "center") {
    case "start":
    case "left":
      return "translate-x-0 text-left";
    case "end":
    case "right":
      return "-translate-x-full text-right";
    default:
      return "-translate-x-1/2 text-center";
  }
};

const cueStyle = (cue: TextTrackCue): CSSProperties => {
  const subtitleCue = cue as SubtitleCue;
  const cueLine = typeof subtitleCue.line === "number" ? subtitleCue.line : null;
  const cuePosition = typeof subtitleCue.position === "number" ? subtitleCue.position : 50;
  const cueSize = typeof subtitleCue.size === "number" ? subtitleCue.size : null;
  const style: CSSProperties = {
    left: `${cuePosition}%`,
    width: cueSize != null ? `${cueSize}%` : undefined,
    maxWidth: cueSize != null ? undefined : "min(90%, 60rem)",
  };

  if (cueLine == null) {
    style.bottom = 0;
    return style;
  }

  if (!subtitleCue.snapToLines) {
    style.top = `${Math.max(0, Math.min(100, cueLine))}%`;
    return style;
  }

  if (cueLine >= 0) {
    style.top = `calc(${cueLine} * var(--player-subtitle-line-step))`;
    return style;
  }

  style.bottom = `calc(${Math.abs(cueLine)} * var(--player-subtitle-line-step))`;
  return style;
};

const cueMarkup = (cue: TextTrackCue) => {
  const html = cueHtmlToString(cue);
  if (html) return html;
  return escapeCueText(cueText(cue)).replaceAll("\n", "<br />");
};

const readActiveSubtitleCues = (track: TextTrack | null): TextTrackCue[] => {
  if (!track?.activeCues?.length) return [];
  return Array.from(track.activeCues);
};

export const PlayerSubtitleOverlay: FC = () => {
  const showControls = usePlayerContext((ctx) => ctx.controls.showControls);
  const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
  const videoElement = usePlayerVideoElement();
  const [activeSubtitleCues, setActiveSubtitleCues] = useState<TextTrackCue[]>([]);

  useEffect(() => {
    if (!videoElement) {
      setActiveSubtitleCues([]);
      return;
    }

    const textTracks = videoElement.textTracks;
    let activeTrack: TextTrack | null = null;

    const getSubtitleTrack = () => {
      for (let index = 0; index < textTracks.length; index++) {
        const track = textTracks[index];
        if (track?.kind === "subtitles") {
          return track;
        }
      }
      return null;
    };

    const syncActiveCues = () => {
      if (activeTrack) {
        activeTrack.mode = "hidden";
      }
      setActiveSubtitleCues(readActiveSubtitleCues(activeTrack));
    };

    const syncActiveTrack = () => {
      const nextTrack = getSubtitleTrack();
      if (activeTrack === nextTrack) {
        syncActiveCues();
        return;
      }

      if (activeTrack) {
        activeTrack.removeEventListener("cuechange", syncActiveCues);
      }

      activeTrack = nextTrack;
      if (!activeTrack) {
        setActiveSubtitleCues([]);
        return;
      }

      activeTrack.mode = "hidden";
      syncActiveCues();
      activeTrack.addEventListener("cuechange", syncActiveCues);
    };

    syncActiveTrack();
    textTracks.addEventListener("change", syncActiveTrack);
    textTracks.addEventListener("addtrack", syncActiveTrack);
    textTracks.addEventListener("removetrack", syncActiveTrack);
    videoElement.addEventListener("loadeddata", syncActiveTrack);
    videoElement.addEventListener("seeked", syncActiveCues);

    return () => {
      if (activeTrack) {
        activeTrack.removeEventListener("cuechange", syncActiveCues);
      }
      textTracks.removeEventListener("change", syncActiveTrack);
      textTracks.removeEventListener("addtrack", syncActiveTrack);
      textTracks.removeEventListener("removetrack", syncActiveTrack);
      videoElement.removeEventListener("loadeddata", syncActiveTrack);
      videoElement.removeEventListener("seeked", syncActiveCues);
      setActiveSubtitleCues([]);
    };
  }, [videoElement]);

  if (activeSubtitleCues.length === 0) return null;

  return (
    <div
      className={cn(
        "pointer-events-none absolute inset-x-0 z-20 transition-[top,bottom] duration-300",
        isFullscreen
          ? showControls
            ? "top-20 bottom-28"
            : "top-5 bottom-5"
          : showControls
            ? "top-12 bottom-22"
            : "top-5 bottom-5",
      )}
      style={{ ["--player-subtitle-line-step" as string]: isFullscreen ? "2rem" : "1.5rem" }}
      aria-live="polite"
    >
      <div className="relative h-full w-full px-3">
        {activeSubtitleCues.map((cue, index) => (
          <div
            key={cueKey(cue, index)}
            className={cn(
              "absolute transition-[transform,opacity] duration-300 ease-out",
              cueAlignClassName(cue as SubtitleCue),
              showControls ? "translate-y-0" : "translate-y-3",
            )}
            style={cueStyle(cue)}
          >
            <span
              className={cn(
                "inline-block rounded bg-black/70 px-2 py-0.5 text-center font-medium text-white ",
                isFullscreen ? "text-lg md:text-3xl" : "text-sm md:text-base",
              )}
              dangerouslySetInnerHTML={{
                __html: cueMarkup(cue),
              }}
            />
          </div>
        ))}
      </div>
    </div>
  );
};
