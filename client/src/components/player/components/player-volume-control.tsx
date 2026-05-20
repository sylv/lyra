import * as Slider from "@radix-ui/react-slider";
import { Volume1Icon, Volume2Icon, VolumeIcon, VolumeXIcon } from "lucide-react";
import { useState, type FC } from "react";
import { cn } from "../../../lib/utils";
import { setPlayerVolume, togglePlayerMute, usePlayerStore } from "../store/player-store";
import { useControlsOverride } from "../store/player-controls-store";

export const PlayerVolumeControl: FC<{ buttonClassName?: string }> = ({ buttonClassName }) => {
  const [showSlider, setShowSlider] = useState(false);
  const volume = usePlayerStore((state) => state.volume);
  const muted = usePlayerStore((state) => state.muted);
  useControlsOverride(showSlider);

  const icon =
    muted || volume === 0 ? (
      <VolumeXIcon className="size-6" />
    ) : volume < 0.33 ? (
      <VolumeIcon className="size-6" />
    ) : volume < 0.66 ? (
      <Volume1Icon className="size-6" />
    ) : (
      <Volume2Icon className="size-6" />
    );

  return (
    <div
      className="flex items-center overflow-hidden"
      onMouseEnter={() => setShowSlider(true)}
      onMouseLeave={() => setShowSlider(false)}
    >
      <button
        aria-label={muted ? "Unmute" : "Mute"}
        className={buttonClassName}
        onClick={(event) => {
          event.stopPropagation();
          togglePlayerMute();
        }}
      >
        {icon}
      </button>
      <div
        className={cn(
          "grid transition-[grid-template-columns,margin,opacity] duration-200 ease-out",
          showSlider ? "ml-1 grid-cols-[1fr] opacity-100" : "pointer-events-none ml-0 grid-cols-[0fr] opacity-0",
        )}
      >
        <div className="overflow-hidden pr-2">
          <Slider.Root
            className="relative flex h-10 w-20 cursor-pointer items-center"
            value={[muted ? 0 : volume]}
            max={1}
            step={0.05}
            onValueChange={(nextValue) => setPlayerVolume(nextValue[0] ?? 0)}
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
