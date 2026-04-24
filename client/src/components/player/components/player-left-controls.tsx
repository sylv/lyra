import { PauseIcon, PlayIcon, SkipBackIcon, SkipForwardIcon } from "lucide-react";
import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import { usePlayerCommands } from "../hooks/use-player-commands";
import { usePlayerRuntimeStore } from "../player-runtime-store";
import { PlayerButton } from "../ui/player-button";
import { PlayerVolumeControl } from "./player-volume-control";

export const PlayerNavigationFragment = graphql(`
  fragment PlayerNavigation on Node {
    id
    ...PlayerItemCard
  }
`);

export const PlayerLeftControls: FC<{
  previousPlayable: FragmentType<typeof PlayerNavigationFragment> | null;
  nextPlayable: FragmentType<typeof PlayerNavigationFragment> | null;
  onHoverPreviewChange: (value: "previous" | "next" | null) => void;
}> = ({ previousPlayable, nextPlayable, onHoverPreviewChange }) => {
  const playing = usePlayerRuntimeStore((state) => state.playing);
  const { switchItem, togglePlaying } = usePlayerCommands();
  const previous = previousPlayable ? unmask(PlayerNavigationFragment, previousPlayable) : null;
  const next = nextPlayable ? unmask(PlayerNavigationFragment, nextPlayable) : null;

  return (
    <div className="flex items-center gap-1 rounded-full bg-black/30 p-1">
      <PlayerButton aria-label={playing ? "Pause" : "Play"} onClick={() => void togglePlaying()}>
        {playing ? <PauseIcon className="size-6 text-white" /> : <PlayIcon className="size-6 text-white" />}
      </PlayerButton>
      <div onMouseEnter={() => onHoverPreviewChange("previous")} onMouseLeave={() => onHoverPreviewChange(null)}>
        <PlayerButton
          aria-label="Previous item"
          disabled={!previous}
          onClick={(event) => {
            event.stopPropagation();
            if (!previous) return;
            void switchItem(previous.id);
          }}
        >
          <SkipBackIcon className="size-5" />
        </PlayerButton>
      </div>
      <div onMouseEnter={() => onHoverPreviewChange("next")} onMouseLeave={() => onHoverPreviewChange(null)}>
        <PlayerButton
          aria-label="Next item"
          disabled={!next}
          onClick={(event) => {
            event.stopPropagation();
            if (!next) return;
            void switchItem(next.id);
          }}
        >
          <SkipForwardIcon className="size-5" />
        </PlayerButton>
      </div>
      <PlayerVolumeControl />
    </div>
  );
};
