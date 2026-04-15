import * as Slider from "@radix-ui/react-slider";
import { Volume1Icon, Volume2Icon, VolumeIcon, VolumeXIcon } from "lucide-react";
import { useState, type FC } from "react";
import { cn } from "../../../lib/utils";
import { usePlayerContext } from "../player-context";
import { PlayerButton } from "./player-button";

export const PlayerVolumeControl: FC = () => {
  const [showSlider, setShowSlider] = useState(false);
  const volume = usePlayerContext((ctx) => ctx.preferences.volume);
  const isMuted = usePlayerContext((ctx) => ctx.preferences.isMuted);
  const toggleMute = usePlayerContext((ctx) => ctx.actions.toggleMute);
  const setVolume = usePlayerContext((ctx) => ctx.actions.setVolume);
  const showControlsTemporarily = usePlayerContext((ctx) => ctx.actions.showControlsTemporarily);
  const beginControlsInteraction = usePlayerContext((ctx) => ctx.actions.beginControlsInteraction);
  const endControlsInteraction = usePlayerContext((ctx) => ctx.actions.endControlsInteraction);

  const getVolumeIcon = () => {
    if (isMuted || volume === 0) return <VolumeXIcon className="size-5" />;
    if (volume < 0.33) return <VolumeIcon className="size-5" />;
    if (volume < 0.66) return <Volume1Icon className="size-5" />;
    return <Volume2Icon className="size-5" />;
  };

  return (
    <div
      className="relative flex items-center"
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
          "absolute left-full flex items-center transition-all duration-200",
          showSlider ? "translate-x-0 opacity-100" : "-translate-x-2 pointer-events-none opacity-0",
        )}
      >
        <div className="flex items-center px-2 py-6">
          <Slider.Root
            className="relative flex h-5 w-20 cursor-pointer items-center"
            value={[isMuted ? 0 : volume]}
            max={1}
            step={0.05}
            onValueChange={(value) => {
              showControlsTemporarily();
              setVolume(value[0] ?? 0);
            }}
            onPointerDown={beginControlsInteraction}
            onPointerUp={endControlsInteraction}
            onPointerCancel={endControlsInteraction}
            onLostPointerCapture={endControlsInteraction}
          >
            <Slider.Track className="relative h-1 grow rounded-full bg-white/20">
              <Slider.Range className="absolute h-full rounded-full bg-white" />
            </Slider.Track>
            <Slider.Thumb className="block size-3 rounded-full bg-white focus:outline-none focus:ring-2 focus:ring-white/50 hover:bg-white/90" />
          </Slider.Root>
        </div>
      </div>
    </div>
  );
};
