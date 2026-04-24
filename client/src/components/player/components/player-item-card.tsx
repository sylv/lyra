import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import { Image, ImageType, Fragment as ImageAssetFragment } from "../../image";

export const PlayerItemCardFragment = graphql(`
  fragment PlayerItemCard on Node {
    id
    properties {
      displayName
      description
      thumbnailImage {
        ...ImageAsset
      }
      seasonNumber
      episodeNumber
    }
  }
`);

interface PlayerItemCardProps {
  item: FragmentType<typeof PlayerItemCardFragment>;
  onPlay?: () => void;
  onCancel?: () => void;
  progressPercent?: number;
  countdownSeconds?: number;
}

export const PlayerItemCard: FC<PlayerItemCardProps> = ({ item: itemRaw, onPlay, onCancel, progressPercent, countdownSeconds }) => {
  const item = unmask(PlayerItemCardFragment, itemRaw);
  const titleParts: string[] = [];

  if (typeof item.properties.seasonNumber === "number" && typeof item.properties.episodeNumber === "number") {
    titleParts.push(`S${item.properties.seasonNumber}E${item.properties.episodeNumber}`);
  }
  titleParts.push(item.properties.displayName);

  const clampedPercent = Math.max(0, Math.min(100, (progressPercent ?? 0) * 100));
  const buttonLabel =
    countdownSeconds != null && countdownSeconds > 0 ? `Playing in ${Math.ceil(countdownSeconds)}s` : "Play now";

  return (
    <div className="flex items-center">
      <Image
        type={ImageType.Thumbnail}
        asset={item.properties.thumbnailImage as FragmentType<typeof ImageAssetFragment> | null | undefined}
        alt={item.properties.displayName}
        className="h-32 rounded-r-none object-cover"
      />
      <div className="flex h-32 w-[20rem] flex-col gap-3 rounded-md rounded-l-none bg-black p-3 pl-5 shadow-lg">
        <div className="min-h-0 flex-1 overflow-hidden">
          <p className="truncate text-sm font-semibold text-white">{titleParts.join(" ")}</p>
          {item.properties.description ? <p className="line-clamp-3 text-xs text-white/70">{item.properties.description}</p> : null}
        </div>
        {onPlay ? (
          <div className="flex gap-2">
            <button
              type="button"
              className="relative overflow-hidden rounded bg-white/70 px-3 py-1 text-xs font-medium text-black transition-colors hover:bg-white"
              onClick={(event) => {
                event.stopPropagation();
                onPlay();
              }}
            >
              {clampedPercent > 0 ? (
                <div className="pointer-events-none absolute inset-0">
                  <div className="h-full bg-white/90 transition-[width] duration-100 ease-linear" style={{ width: `${clampedPercent}%` }} />
                </div>
              ) : null}
              <span className="relative z-10">{buttonLabel}</span>
            </button>
            {onCancel ? (
              <button
                type="button"
                className="rounded px-3 py-1 text-xs font-medium text-white/60 transition-colors hover:text-white"
                onClick={(event) => {
                  event.stopPropagation();
                  onCancel();
                }}
              >
                Cancel
              </button>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  );
};
