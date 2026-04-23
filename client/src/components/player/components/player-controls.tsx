/* oxlint-disable jsx_a11y/click-events-have-key-events, jsx_a11y/no-static-element-interactions */
import {
  CaptionsIcon,
  FileTextIcon,
  LoaderCircle,
  MaximizeIcon,
  MinimizeIcon,
  ScanSearchIcon,
  PauseIcon,
  PlayIcon,
  SettingsIcon,
  SkipBackIcon,
  SkipForwardIcon,
  SparklesIcon,
} from "lucide-react";
import { useEffect, useMemo, useState, type FC } from "react";
import { useClient, useMutation } from "urql";
import type { FragmentType } from "../../../@generated/gql";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { formatPlayerTime } from "../../../lib/format-player-time";
import { cn } from "../../../lib/utils";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "../../ui/dropdown-menu";
import {
  playerContext,
  setPlayerControls,
  setPlayerPreferences,
  togglePlayerFullscreen,
  usePlayerContext,
} from "../player-context";
import { usePlayerActions } from "../hooks/use-player-actions";
import { DisabledSubtitlesHint, ItemPlaybackQuery as ItemPlaybackQueryDoc, SetPreferredAudio } from "../player-queries";
import { PlayerButton } from "./player-button";
import { PlayerProgressBar, PlayerTimelinePreviewSheetFragment } from "./player-progress-bar";
import { PlayerVolumeControl } from "./player-volume-control";

type PlayableNode = NonNullable<NonNullable<ItemPlaybackQuery["node"]>["previousPlayable"]>;

const useTrackSelection = () => {
  const client = useClient();
  const [, setPreferredAudio] = useMutation(SetPreferredAudio);
  const [, disabledSubtitlesHint] = useMutation(DisabledSubtitlesHint);
  const currentItemId = usePlayerContext((ctx) => ctx.currentItemId);
  const currentFileId = usePlayerContext((ctx) => ctx.watchSession.fileId);
  const audioTrackOptions = usePlayerContext((ctx) => ctx.state.audioTrackOptions);
  const activeSubtitleTrackId = usePlayerContext((ctx) => ctx.state.activeSubtitleTrackId);
  const activeSubtitleRenditionId = usePlayerContext((ctx) => ctx.state.activeSubtitleRenditionId);
  const languageHints = typeof navigator === "undefined" ? [] : [...navigator.languages];

  const refreshPlaybackQuery = () =>
    currentItemId
      ? client
          .query(ItemPlaybackQueryDoc, { itemId: currentItemId, languageHints }, { requestPolicy: "network-only" })
          .toPromise()
      : Promise.resolve(null);

  const onAudioTrackChange = (trackId: number | null) => {
    playerContext.getState().actions.setAudioTrack(trackId);
    if (trackId === null || Number.isNaN(trackId)) return;

    const selectedTrack = audioTrackOptions.find((track) => track.id === trackId);
    if (selectedTrack?.language != null) {
      setPreferredAudio({ language: selectedTrack.language, disposition: null })
        .then((result) => {
          if (result.error) {
            throw result.error;
          }
          return refreshPlaybackQuery();
        })
        .catch((err: unknown) => {
          console.error("failed to save audio preference", err);
        });
    }
  };

  const onSubtitleTrackChange = (trackId: string | null) => {
    const { setSubtitleTrack } = playerContext.getState().actions;
    if (trackId === null || trackId === "") {
      setSubtitleTrack(trackId, { manual: false });
      if (trackId === "" && activeSubtitleTrackId && activeSubtitleRenditionId && currentFileId) {
        disabledSubtitlesHint({
          input: {
            fileId: currentFileId,
            trackId: activeSubtitleTrackId,
            renditionId: activeSubtitleRenditionId,
          },
        }).catch((err: unknown) => {
          console.error("failed to send subtitles-disabled hint", err);
        });
      }
      return;
    }

    void setSubtitleTrack(trackId, { manual: true });
  };

  return { onAudioTrackChange, onSubtitleTrackChange };
};

interface PlayerControlsProps {
  timelinePreviewSheets: FragmentType<typeof PlayerTimelinePreviewSheetFragment>[];
  mode?: "fullscreen" | "mini";
  previousPlayable: PlayableNode | null | undefined;
  nextPlayable: PlayableNode | null | undefined;
  onPreviousItem: () => void;
  onNextItem: () => void;
  dropdownPortalContainer: HTMLElement | null;
}

export const PlayerControls: FC<PlayerControlsProps> = ({
  timelinePreviewSheets,
  mode = "fullscreen",
  previousPlayable,
  nextPlayable,
  onPreviousItem,
  onNextItem,
  dropdownPortalContainer,
}) => {
  const currentTime = usePlayerContext((ctx) => ctx.state.currentTime);
  const duration = usePlayerContext((ctx) => ctx.state.duration);
  const playing = usePlayerContext((ctx) => ctx.state.playing);
  const audioTrackOptions = usePlayerContext((ctx) => ctx.state.audioTrackOptions);
  const selectedAudioTrackId = usePlayerContext((ctx) => ctx.state.selectedAudioTrackId);
  const subtitleTrackOptions = usePlayerContext((ctx) => ctx.state.subtitleTrackOptions);
  const selectedSubtitleTrackId = usePlayerContext((ctx) => ctx.state.selectedSubtitleTrackId);
  const activeSubtitleTrackId = usePlayerContext((ctx) => ctx.state.activeSubtitleTrackId);
  const pendingSubtitleTrackId = usePlayerContext((ctx) => ctx.state.pendingSubtitleTrackId);
  const showControls = usePlayerContext((ctx) => ctx.controls.showControls);
  const isSettingsMenuOpen = usePlayerContext((ctx) => ctx.controls.isSettingsMenuOpen);
  const autoplayNext = usePlayerContext((ctx) => ctx.preferences.autoplayNext);
  const watchSessionMode = usePlayerContext((ctx) => ctx.watchSession.mode);
  const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
  const { togglePlaying } = usePlayerActions();
  const { onAudioTrackChange, onSubtitleTrackChange } = useTrackSelection();
  const [hoveredButton, setHoveredButton] = useState<"previous" | "next" | null>(null);
  const isMini = mode === "mini";

  const hasPreviousItem = !isMini && !!previousPlayable;
  const hasNextItem = !isMini && !!nextPlayable;
  const autoselectSubtitleTrack = subtitleTrackOptions.find((track) => track.autoselect) ?? null;
  const effectiveSelectedSubtitleTrackId =
    selectedSubtitleTrackId === ""
      ? ""
      : (selectedSubtitleTrackId ?? activeSubtitleTrackId ?? autoselectSubtitleTrack?.id ?? "");

  useEffect(() => {
    setPlayerControls({ hoveredCard: hoveredButton, isItemCardOpen: hoveredButton !== null });
  }, [hoveredButton]);

  useEffect(() => {
    return () => {
      setPlayerControls({ hoveredCard: null, isItemCardOpen: false });
    };
  }, []);

  const finishTime = useMemo(() => {
    if (!duration || !currentTime) return null;
    const remainingTimeMs = (duration - currentTime) * 1000;
    const finishDate = new Date(Date.now() + remainingTimeMs);
    return finishDate.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }, [currentTime, duration]);

  return (
    <div
      onClick={(event) => event.stopPropagation()}
      className={cn(
        "group cursor-default transition-opacity duration-300",
        showControls ? "pointer-events-auto opacity-100" : "pointer-events-none opacity-0",
        isMini ? "p-4" : "p-6",
      )}
    >
      <div className={cn("flex justify-between text-white/80", isMini ? "text-xs" : "text-sm")}>
        <span>{formatPlayerTime(currentTime)}</span>
        <span>{formatPlayerTime(duration)}</span>
      </div>

      <PlayerProgressBar compact={isMini} timelinePreviewSheets={timelinePreviewSheets} />

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 rounded-full">
          <PlayerButton aria-label={playing ? "Pause" : "Play"} onClick={togglePlaying}>
            {playing ? <PauseIcon className="size-6 text-white" /> : <PlayIcon className="size-6 text-white" />}
          </PlayerButton>
          {(hasPreviousItem || hasNextItem) && (
            <>
              <div onMouseEnter={() => setHoveredButton("previous")} onMouseLeave={() => setHoveredButton(null)}>
                <PlayerButton
                  aria-label="Previous item"
                  disabled={!hasPreviousItem}
                  onClick={(event) => {
                    event.stopPropagation();
                    if (hasPreviousItem) onPreviousItem();
                  }}
                >
                  <SkipBackIcon className="size-5" />
                </PlayerButton>
              </div>
              <div onMouseEnter={() => setHoveredButton("next")} onMouseLeave={() => setHoveredButton(null)}>
                <PlayerButton
                  aria-label="Next item"
                  disabled={!hasNextItem}
                  onClick={(event) => {
                    event.stopPropagation();
                    if (hasNextItem) onNextItem();
                  }}
                >
                  <SkipForwardIcon className="size-5" />
                </PlayerButton>
              </div>
            </>
          )}
          <PlayerVolumeControl />
        </div>
        <div className="flex items-center gap-4">
          {finishTime && !isMini && <span className="text-sm">Finishes at {finishTime}</span>}
          {!isMini && (
            <DropdownMenu
              open={isSettingsMenuOpen}
              onOpenChange={(open) => setPlayerControls({ isSettingsMenuOpen: open })}
            >
              <DropdownMenuTrigger asChild>
                <PlayerButton
                  className="relative"
                  aria-label="Open player settings"
                  onClick={(event) => {
                    event.stopPropagation();
                  }}
                >
                  <SettingsIcon className="size-5" />
                  {pendingSubtitleTrackId ? (
                    <LoaderCircle className="absolute -top-0.5 -right-0.5 size-3.5 animate-spin rounded-full bg-black" />
                  ) : null}
                </PlayerButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align="end"
                portalContainer={dropdownPortalContainer}
                onClick={(event) => event.stopPropagation()}
                className="z-70 w-56 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40"
              >
                <DropdownMenuSub>
                  <DropdownMenuSubTrigger className="py-2.5 data-[state=open]:bg-zinc-800 focus:bg-zinc-800">
                    Audio
                  </DropdownMenuSubTrigger>
                  <DropdownMenuSubContent className="z-70 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40">
                    {audioTrackOptions.length === 0 ? (
                      <DropdownMenuItem className="py-2.5" disabled>
                        No audio tracks
                      </DropdownMenuItem>
                    ) : (
                      <DropdownMenuRadioGroup
                        value={selectedAudioTrackId?.toString() ?? "auto"}
                        onValueChange={(value) =>
                          value === "auto" ? onAudioTrackChange(null) : onAudioTrackChange(Number.parseInt(value, 10))
                        }
                      >
                        <DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="auto">
                          Auto
                        </DropdownMenuRadioItem>
                        {audioTrackOptions.map((track) => (
                          <DropdownMenuRadioItem
                            className="py-2.5 focus:bg-zinc-800"
                            key={track.id}
                            value={track.id.toString()}
                          >
                            {track.label}
                          </DropdownMenuRadioItem>
                        ))}
                      </DropdownMenuRadioGroup>
                    )}
                  </DropdownMenuSubContent>
                </DropdownMenuSub>
                <DropdownMenuSub>
                  <DropdownMenuSubTrigger className="py-2.5 data-[state=open]:bg-zinc-800 focus:bg-zinc-800">
                    Subtitles
                  </DropdownMenuSubTrigger>
                  <DropdownMenuSubContent className="z-70 w-64 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40">
                    {subtitleTrackOptions.length === 0 ? (
                      <DropdownMenuItem className="py-2.5" disabled>
                        No subtitles
                      </DropdownMenuItem>
                    ) : (
                      <DropdownMenuRadioGroup
                        value={effectiveSelectedSubtitleTrackId === "" ? "off" : effectiveSelectedSubtitleTrackId}
                        onValueChange={(value) => {
                          if (value === "off") onSubtitleTrackChange("");
                          else onSubtitleTrackChange(value);
                        }}
                      >
                        <DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="off">
                          Off
                        </DropdownMenuRadioItem>
                        {subtitleTrackOptions.map((track) => (
                          <DropdownMenuRadioItem
                            className="py-2.5 focus:bg-zinc-800"
                            key={track.id}
                            value={track.id}
                            onSelect={(event) => event.preventDefault()}
                          >
                            <div className="min-w-0">
                              <span className="block truncate">{track.label}</span>
                              {effectiveSelectedSubtitleTrackId === track.id ? (
                                <span className="mt-1 flex items-center gap-2 text-xs text-zinc-400">
                                  <span className="shrink-0">
                                    {pendingSubtitleTrackId === track.id ? (
                                      <LoaderCircle className="size-3.5 animate-spin" />
                                    ) : (
                                      subtitleSourceIcon(track.renditionType)
                                    )}
                                  </span>
                                  <span className="truncate">{track.displayInfo}</span>
                                </span>
                              ) : null}
                            </div>
                          </DropdownMenuRadioItem>
                        ))}
                      </DropdownMenuRadioGroup>
                    )}
                  </DropdownMenuSubContent>
                </DropdownMenuSub>
                <DropdownMenuSeparator className="bg-zinc-700" />
                <DropdownMenuCheckboxItem
                  className="py-2.5 focus:bg-zinc-800"
                  checked={autoplayNext}
                  disabled={watchSessionMode === "SYNCED"}
                  onCheckedChange={(checked) => setPlayerPreferences({ autoplayNext: !!checked })}
                >
                  {watchSessionMode === "SYNCED" ? "Autoplay (disabled in synced sessions)" : "Autoplay"}
                </DropdownMenuCheckboxItem>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
          <PlayerButton
            aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
            onClick={(event) => {
              event.stopPropagation();
              togglePlayerFullscreen();
            }}
          >
            {isFullscreen ? <MinimizeIcon className="size-5" /> : <MaximizeIcon className="size-5" />}
          </PlayerButton>
        </div>
      </div>
    </div>
  );
};

const subtitleSourceIcon = (source: "DIRECT" | "CONVERTED" | "OCR" | "GENERATED") => {
  switch (source) {
    case "DIRECT":
      return <CaptionsIcon className="size-3.5" />;
    case "CONVERTED":
      return <FileTextIcon className="size-3.5" />;
    case "OCR":
      return <ScanSearchIcon className="size-3.5" />;
    case "GENERATED":
      return <SparklesIcon className="size-3.5" />;
  }
};
