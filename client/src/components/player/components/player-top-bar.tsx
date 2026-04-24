import { XIcon } from "lucide-react";
import type { FC } from "react";
import type { FragmentType } from "../../../@generated/gql";
import { cn } from "../../../lib/utils";
import { clearPlayerMedia, usePlayerRuntimeStore } from "../player-runtime-store";
import { usePlayerVisibility } from "../player-visibility";
import { PlayerButton } from "../ui/player-button";
import { PlayerMetadata, PlayerMetadataFragment } from "./player-metadata";
import { PlayerSessionMenu } from "./player-session-menu";

export const PlayerTopBar: FC<{ media: FragmentType<typeof PlayerMetadataFragment> | null; portalContainer: HTMLElement | null }> = ({
  media,
  portalContainer,
}) => {
  const { showControls } = usePlayerVisibility();
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);

  return (
    <div
      className={cn(
        "pointer-events-none flex items-center justify-between transition-opacity duration-300",
        showControls ? "opacity-100" : "opacity-0",
        isFullscreen ? "p-3" : "p-2.5",
      )}
    >
      <div className="pointer-events-auto flex items-center gap-3">
        {media ? <PlayerMetadata media={media} /> : null}
      </div>
      <div className="pointer-events-auto flex items-center gap-1 rounded-full bg-black/30 p-1 text-white">
        {media ? <PlayerSessionMenu portalContainer={portalContainer} /> : null}
        <PlayerButton
          aria-label="Close player"
          onClick={(event) => {
            event.stopPropagation();
            clearPlayerMedia();
          }}
        >
          <XIcon className={isFullscreen ? "size-6" : "size-5"} />
        </PlayerButton>
      </div>
    </div>
  );
};
