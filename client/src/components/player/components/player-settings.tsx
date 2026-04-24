import { CaptionsIcon, FileTextIcon, LoaderCircle, ScanSearchIcon, SettingsIcon, SparklesIcon } from "lucide-react";
import { useState, type FC } from "react";
import { useMutation } from "urql";
import { graphql } from "../../../@generated/gql";
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
import { usePlayerCommands } from "../hooks/use-player-commands";
import { usePlayerOptionsStore } from "../player-options-store";
import { usePlayerRuntimeStore } from "../player-runtime-store";
import { usePlayerSession } from "../player-session";
import { useShowControlsLock } from "../player-visibility";
import { PlayerButton } from "../ui/player-button";

const SetPreferredAudio = graphql(`
  mutation SetPreferredAudio($language: String, $disposition: TrackDispositionPreference) {
    setPreferredAudio(language: $language, disposition: $disposition) {
      id
      preferredAudioLanguage
      preferredAudioDisposition
    }
  }
`);

const DisabledSubtitlesHint = graphql(`
  mutation DisabledSubtitlesHint($input: DisabledSubtitlesHintInput!) {
    disabledSubtitlesHint(input: $input)
  }
`);

const subtitleSourceIcon = (source: string) => {
  switch (source) {
    case "OCR":
      return <ScanSearchIcon className="size-3.5" />;
    case "GENERATED":
      return <SparklesIcon className="size-3.5" />;
    case "CONVERTED":
      return <CaptionsIcon className="size-3.5" />;
    default:
      return <FileTextIcon className="size-3.5" />;
  }
};

export const PlayerSettings: FC<{ portalContainer: HTMLElement | null }> = ({ portalContainer }) => {
  const { setAudioTrack, setSubtitleTrack, setVideoRendition } = usePlayerCommands();
  const { session } = usePlayerSession();
  const currentItemId = usePlayerRuntimeStore((state) => state.currentItemId);
  const videoRenditionOptions = usePlayerRuntimeStore((state) => state.videoRenditionOptions);
  const selectedVideoRenditionId = usePlayerRuntimeStore((state) => state.selectedVideoRenditionId);
  const audioTrackOptions = usePlayerRuntimeStore((state) => state.audioTrackOptions);
  const selectedAudioTrackId = usePlayerRuntimeStore((state) => state.selectedAudioTrackId);
  const subtitleTrackOptions = usePlayerRuntimeStore((state) => state.subtitleTrackOptions);
  const selectedSubtitleTrackId = usePlayerRuntimeStore((state) => state.selectedSubtitleTrackId);
  const activeSubtitleTrackId = usePlayerRuntimeStore((state) => state.activeSubtitleTrackId);
  const activeSubtitleRenditionId = usePlayerRuntimeStore((state) => state.activeSubtitleRenditionId);
  const pendingSubtitleTrackId = usePlayerRuntimeStore((state) => state.pendingSubtitleTrackId);
  const autoplayNext = usePlayerOptionsStore((state) => state.autoplayNext);
  const setAutoplayNext = usePlayerOptionsStore((state) => state.setAutoplayNext);
  const [open, setOpen] = useState(false);
  const [, setPreferredAudio] = useMutation(SetPreferredAudio);
  const [, disabledSubtitlesHint] = useMutation(DisabledSubtitlesHint);
  useShowControlsLock(open || pendingSubtitleTrackId != null);
  const showVideoSettings = import.meta.env.DEV;

  const autoselectSubtitleTrack = subtitleTrackOptions.find((track) => track.autoselect) ?? null;
  const effectiveSelectedSubtitleTrackId =
    selectedSubtitleTrackId === ""
      ? ""
      : (selectedSubtitleTrackId ?? activeSubtitleTrackId ?? autoselectSubtitleTrack?.id ?? "");

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <PlayerButton
          aria-label="Open player settings"
          className="relative"
          onClick={(event) => {
            event.stopPropagation();
          }}
        >
          <SettingsIcon className="size-5" />
          {pendingSubtitleTrackId ? (
            <LoaderCircle className="absolute -right-0.5 -top-0.5 size-3.5 animate-spin rounded-full bg-black" />
          ) : null}
        </PlayerButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        portalContainer={portalContainer}
        className="z-70 w-56 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40"
        onClick={(event) => event.stopPropagation()}
      >
        {showVideoSettings ? (
          <DropdownMenuSub>
            <DropdownMenuSubTrigger className="py-2.5 data-[state=open]:bg-zinc-800 focus:bg-zinc-800">Video</DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="z-70 w-64 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40">
              {videoRenditionOptions.length === 0 ? (
                <DropdownMenuItem className="py-2.5" disabled>
                  No video renditions
                </DropdownMenuItem>
              ) : (
                <DropdownMenuRadioGroup
                  value={selectedVideoRenditionId ?? "auto"}
                  onValueChange={(value) => {
                    setVideoRendition(value === "auto" ? null : value);
                  }}
                >
                  <DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="auto">
                    Auto
                  </DropdownMenuRadioItem>
                  {videoRenditionOptions.map((rendition) => (
                    <DropdownMenuRadioItem
                      className="py-2.5 focus:bg-zinc-800"
                      key={rendition.id}
                      value={rendition.id}
                      onSelect={(event) => event.preventDefault()}
                    >
                      <div className="min-w-0">
                        <span className="block truncate">{rendition.label}</span>
                        <span className="mt-1 block truncate text-xs text-zinc-400">{rendition.displayInfo}</span>
                      </div>
                    </DropdownMenuRadioItem>
                  ))}
                </DropdownMenuRadioGroup>
              )}
            </DropdownMenuSubContent>
          </DropdownMenuSub>
        ) : null}
        <DropdownMenuSub>
          <DropdownMenuSubTrigger className="py-2.5 data-[state=open]:bg-zinc-800 focus:bg-zinc-800">Audio</DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="z-70 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40">
            {audioTrackOptions.length === 0 ? (
              <DropdownMenuItem className="py-2.5" disabled>
                No audio tracks
              </DropdownMenuItem>
            ) : (
              <DropdownMenuRadioGroup
                value={selectedAudioTrackId?.toString() ?? "auto"}
                onValueChange={(value) => {
                  const trackId = value === "auto" ? null : Number.parseInt(value, 10);
                  setAudioTrack(trackId);
                  if (trackId == null || Number.isNaN(trackId)) return;

                  const selectedTrack = audioTrackOptions.find((track) => track.id === trackId);
                  if (selectedTrack?.language != null) {
                    void setPreferredAudio({ language: selectedTrack.language, disposition: null }).catch((error) =>
                      console.error("failed to save audio preference", error),
                    );
                  }
                }}
              >
                <DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="auto">
                  Auto
                </DropdownMenuRadioItem>
                {audioTrackOptions.map((track) => (
                  <DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" key={track.id} value={track.id.toString()}>
                    {track.label}
                  </DropdownMenuRadioItem>
                ))}
              </DropdownMenuRadioGroup>
            )}
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        <DropdownMenuSub>
          <DropdownMenuSubTrigger className="py-2.5 data-[state=open]:bg-zinc-800 focus:bg-zinc-800">Subtitles</DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="z-70 w-64 border-zinc-700 bg-black text-zinc-100 shadow-lg shadow-black/40">
            {subtitleTrackOptions.length === 0 ? (
              <DropdownMenuItem className="py-2.5" disabled>
                No subtitles
              </DropdownMenuItem>
            ) : (
              <DropdownMenuRadioGroup
                value={effectiveSelectedSubtitleTrackId === "" ? "off" : effectiveSelectedSubtitleTrackId}
                onValueChange={(value) => {
                  if (value === "off") {
                    setSubtitleTrack("");
                    if (activeSubtitleTrackId && activeSubtitleRenditionId && session.fileId) {
                      void disabledSubtitlesHint({
                        input: {
                          fileId: session.fileId,
                          trackId: activeSubtitleTrackId,
                          renditionId: activeSubtitleRenditionId,
                        },
                      }).catch((error) => console.error("failed to send subtitles-disabled hint", error));
                    }
                    return;
                  }
                  setSubtitleTrack(value);
                }}
              >
                <DropdownMenuRadioItem className="py-2.5 focus:bg-zinc-800" value="off">
                  Off
                </DropdownMenuRadioItem>
                {subtitleTrackOptions.map((track) => (
                  <DropdownMenuRadioItem
                    key={track.id}
                    value={track.id}
                    className="py-2.5 focus:bg-zinc-800"
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
        <DropdownMenuSeparator />
        <DropdownMenuCheckboxItem
          checked={autoplayNext}
          disabled={session.mode === "SYNCED" || currentItemId == null}
          onCheckedChange={(checked) => {
            setAutoplayNext(checked === true);
          }}
        >
          Autoplay next episode
        </DropdownMenuCheckboxItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
