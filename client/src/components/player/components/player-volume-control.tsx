import * as Slider from "@radix-ui/react-slider";
import { Volume1Icon, Volume2Icon, VolumeIcon, VolumeXIcon } from "lucide-react";
import { useState, type FC } from "react";
import { cn } from "../../../lib/utils";
import { usePlayerCommands } from "../hooks/use-player-commands";
import { usePlayerOptionsStore } from "../player-options-store";
import { usePlayerVisibility, useShowControlsLock } from "../player-visibility";
import { PlayerButton } from "../ui/player-button";

export const PlayerVolumeControl: FC = () => {
  const [showSlider, setShowSlider] = useState(false);
  const volume = usePlayerOptionsStore((state) => state.volume);
  const isMuted = usePlayerOptionsStore((state) => state.isMuted);
  const { toggleMute, setVolume } = usePlayerCommands();
  const { showControlsTemporarily } = usePlayerVisibility();
  useShowControlsLock(showSlider);

  const getVolumeIcon = () => {
    if (isMuted || volume === 0) return <VolumeXIcon className="size-5" />;
    if (volume < 0.33) return <VolumeIcon className="size-5" />;
    if (volume < 0.66) return <Volume1Icon className="size-5" />;
    return <Volume2Icon className="size-5" />;
  };

  return (
    <div
      className="flex items-center overflow-hidden"
      onMouseEnter={() => {
        setShowSlider(true);
        showControlsTemporarily();
      }}
      onMouseLeave={() => setShowSlider(false)}
    >
      <PlayerButton
        aria-label={isMuted ? "Unmute" : "Mute"}
        onClick={(event) => {
          event.stopPropagation();
          showControlsTemporarily();
          toggleMute();
        }}
      >
        {getVolumeIcon()}
      </PlayerButton>
      <div
        className={cn(
          "grid transition-[grid-template-columns,margin,opacity] duration-200 ease-out",
          showSlider ? "ml-1 grid-cols-[1fr] opacity-100" : "pointer-events-none ml-0 grid-cols-[0fr] opacity-0",
        )}
      >
        <div className="overflow-hidden pr-2">
          <Slider.Root
            className="relative flex h-10 w-20 cursor-pointer items-center"
            value={[isMuted ? 0 : volume]}
            max={1}
            step={0.05}
            onValueChange={(nextValue) => {
              showControlsTemporarily();
              setVolume(nextValue[0] ?? 0);
            }}
          >
            <Slider.Track className="relative h-1 grow rounded-full bg-white/20">
              <Slider.Range className="absolute h-full rounded-full bg-white" />
            </Slider.Track>
            <Slider.Thumb className="block size-3 rounded-full bg-white hover:bg-white/90 focus:outline-none focus:ring-2 focus:ring-white/50" />
          </Slider.Root>
        </div>
      </div>
    </div>
  );
};
