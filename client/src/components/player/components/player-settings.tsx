import { CaptionsIcon, GaugeIcon, LoaderCircleIcon, SettingsIcon, Volume2Icon, VideoIcon } from "lucide-react";
import { useState, type FC } from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "../../ui/dropdown-menu";
import { useControlsOverride } from "../store/player-controls-store";
import { PlayerState, usePlayerStore } from "../store/player-store";

const menuClassName =
  "z-[80] overflow-visible border-0 bg-black/92 text-xs text-zinc-100 shadow-lg shadow-black/50 backdrop-blur-xl";
const itemClassName =
  "py-1.5 pr-2 pl-2 text-xs focus:bg-white/10 data-[state=checked]:bg-white/6 [&>span:first-child]:hidden";
const subTriggerClassName = "py-1.5 px-2 text-xs focus:bg-white/10 data-[state=open]:bg-white/6";
const speedOptions = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];
type SettingsSubmenu = "video" | "audio" | "speed" | "subtitles";

export const PlayerSettings: FC<{ buttonClassName?: string; portalContainer: HTMLElement | null }> = ({
  buttonClassName,
  portalContainer,
}) => {
  const [open, setOpen] = useState(false);
  const [activeSubmenu, setActiveSubmenu] = useState<SettingsSubmenu | null>(null);
  const status = usePlayerStore((state) => state.status);
  const selectedVideoRenditionPairId = usePlayerStore((state) => state.selectedVideoRenditionPairId);
  const selectedAudioTrackId = usePlayerStore((state) => state.selectedAudioTrackId);
  const selectedSubtitleTrackId = usePlayerStore((state) => state.selectedSubtitleTrackId);
  const pendingSubtitleTrackId = usePlayerStore((state) => state.pendingSubtitleTrackId);
  const videoRenditionOptions = usePlayerStore((state) => state.videoRenditionOptions);
  const audioTrackOptions = usePlayerStore((state) => state.audioTrackOptions);
  const playbackRate = usePlayerStore((state) => state.playbackRate);
  useControlsOverride(open || pendingSubtitleTrackId != null);

  if (status.state !== PlayerState.Mounted) return null;

  const effectiveSubtitleTrackId =
    selectedSubtitleTrackId === ""
      ? ""
      : (selectedSubtitleTrackId ?? status.subtitleTracks.find((track) => track.autoselect)?.sourceTrackId ?? "");
  const supportedVideoRenditions = videoRenditionOptions.filter(
    ({ compatibility }) => compatibility === "probably" || compatibility === "maybe",
  );
  const effectiveVideoRenditionPairId =
    selectedVideoRenditionPairId ?? supportedVideoRenditions[0]?.rendition.pairId ?? "";
  const supportedAudioTracks = audioTrackOptions.filter(({ supportedRenditions }) => supportedRenditions.length > 0);
  const effectiveAudioTrackId =
    selectedAudioTrackId ?? status.audioTrack?.sourceTrackId ?? supportedAudioTracks[0]?.track.sourceTrackId ?? "";
  const setSubmenuOpen = (submenu: SettingsSubmenu, nextOpen: boolean) => {
    setActiveSubmenu((current) => (nextOpen ? submenu : current === submenu ? null : current));
  };

  return (
    <DropdownMenu
      open={open}
      onOpenChange={(nextOpen) => {
        setOpen(nextOpen);
        if (!nextOpen) setActiveSubmenu(null);
      }}
    >
      <DropdownMenuTrigger asChild>
        <button
          aria-label="Open player settings"
          className={buttonClassName}
          onClick={(event) => {
            event.stopPropagation();
          }}
        >
          <SettingsIcon className="size-6" />
          {pendingSubtitleTrackId ? (
            <LoaderCircleIcon className="absolute -right-0.5 -top-0.5 size-3.5 animate-spin rounded-full bg-black" />
          ) : null}
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        portalContainer={portalContainer}
        className={`${menuClassName} w-52`}
        collisionPadding={32}
        onClick={(event) => event.stopPropagation()}
      >
        <DropdownMenuSub
          open={activeSubmenu === "video"}
          onOpenChange={(nextOpen) => setSubmenuOpen("video", nextOpen)}
        >
          <DropdownMenuSubTrigger className={subTriggerClassName}>
            <VideoIcon className="mr-2 size-4" />
            Video
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent
            sideOffset={12}
            collisionPadding={32}
            className={`${menuClassName} max-h-[calc(100dvh-4rem)] w-60 overflow-y-auto`}
          >
            <DropdownMenuRadioGroup
              value={effectiveVideoRenditionPairId}
              onValueChange={(value) => {
                usePlayerStore.setState({ selectedVideoRenditionPairId: value });
              }}
            >
              {supportedVideoRenditions.map(({ track, rendition }) => (
                <DropdownMenuRadioItem
                  key={rendition.pairId}
                  className={itemClassName}
                  value={rendition.pairId}
                  onSelect={(event) => event.preventDefault()}
                >
                  <div className="min-w-0">
                    <span className="block truncate">{track.displayName}</span>
                    <span className="mt-1 block truncate text-xs text-zinc-400">{rendition.displayInfo}</span>
                  </div>
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        <DropdownMenuSub
          open={activeSubmenu === "audio"}
          onOpenChange={(nextOpen) => setSubmenuOpen("audio", nextOpen)}
        >
          <DropdownMenuSubTrigger className={subTriggerClassName}>
            <Volume2Icon className="mr-2 size-4" />
            Audio
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent
            sideOffset={12}
            collisionPadding={32}
            className={`${menuClassName} max-h-[calc(100dvh-4rem)] w-56 overflow-y-auto`}
          >
            <DropdownMenuRadioGroup
              value={effectiveAudioTrackId}
              onValueChange={(value) => {
                usePlayerStore.setState({ selectedAudioTrackId: value });
              }}
            >
              {supportedAudioTracks.map(({ track, supportedRenditions }) => (
                <DropdownMenuRadioItem key={track.sourceTrackId} className={itemClassName} value={track.sourceTrackId}>
                  <div className="min-w-0">
                    <span className="block truncate">{track.displayName}</span>
                    <span className="mt-1 block truncate text-xs text-zinc-400">
                      {[track.languageBcp47, supportedRenditions[0]?.rendition.displayInfo].filter(Boolean).join(" · ")}
                    </span>
                  </div>
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        <DropdownMenuSub
          open={activeSubmenu === "speed"}
          onOpenChange={(nextOpen) => setSubmenuOpen("speed", nextOpen)}
        >
          <DropdownMenuSubTrigger className={subTriggerClassName}>
            <GaugeIcon className="mr-2 size-4" />
            Speed
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent
            sideOffset={12}
            collisionPadding={32}
            className={`${menuClassName} max-h-[calc(100dvh-4rem)] w-36 overflow-y-auto`}
          >
            <DropdownMenuRadioGroup
              value={playbackRate.toString()}
              onValueChange={(value) => {
                const nextRate = Number.parseFloat(value);
                if (!Number.isFinite(nextRate)) return;
                usePlayerStore.setState({ playbackRate: nextRate });
              }}
            >
              {speedOptions.map((speed) => (
                <DropdownMenuRadioItem key={speed} className={itemClassName} value={speed.toString()}>
                  {speed === 1 ? "Normal" : `${speed}x`}
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
        <DropdownMenuSeparator className="bg-white/10" />
        <DropdownMenuSub
          open={activeSubmenu === "subtitles"}
          onOpenChange={(nextOpen) => setSubmenuOpen("subtitles", nextOpen)}
        >
          <DropdownMenuSubTrigger className={subTriggerClassName}>
            <CaptionsIcon className="mr-2 size-4" />
            Subtitles
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent
            sideOffset={12}
            collisionPadding={32}
            className={`${menuClassName} max-h-[calc(100dvh-4rem)] w-64 overflow-y-auto`}
          >
            <DropdownMenuRadioGroup
              value={effectiveSubtitleTrackId === "" ? "off" : effectiveSubtitleTrackId}
              onValueChange={(value) => {
                usePlayerStore.setState({ selectedSubtitleTrackId: value === "off" ? "" : value });
              }}
            >
              <DropdownMenuRadioItem className={itemClassName} value="off">
                Off
              </DropdownMenuRadioItem>
              {status.subtitleTracks.map((track) => (
                <DropdownMenuRadioItem
                  key={track.sourceTrackId}
                  value={track.sourceTrackId}
                  className={itemClassName}
                  onSelect={(event) => event.preventDefault()}
                >
                  <div className="flex min-w-0 flex-1 items-center gap-3">
                    <div className="min-w-0 flex-1">
                      <span className="block truncate">{track.displayName}</span>
                      <span className="mt-1 block truncate text-xs text-zinc-400">
                        {[track.languageBcp47, track.kind, track.renditions[0]?.displayInfo]
                          .filter(Boolean)
                          .join(" · ")}
                      </span>
                    </div>
                    {pendingSubtitleTrackId === track.sourceTrackId ? (
                      <LoaderCircleIcon className="size-4 shrink-0 animate-spin" />
                    ) : null}
                  </div>
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
