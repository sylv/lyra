import type { FC } from "react";
import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForNode } from "../lib/getPathForMedia";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface SeasonCardProps {
  season: FragmentType<typeof Fragment>;
}

const Fragment = graphql(`
  fragment SeasonCard on Node {
    id
    unavailableAt
    properties {
      displayName
      seasonNumber
      posterImage {
        ...ImageAsset
      }
      thumbnailImage {
        ...ImageAsset
      }
      firstAired
      lastAired
    }
    currentPlayable {
      id
      watchProgress {
        id
        progressPercent
        completed
        updatedAt
      }
    }
    unplayedCount
    episodeCount
    ...GetPathForNode
  }
`);

export const SeasonCard: FC<SeasonCardProps> = ({ season: seasonRaw }) => {
  const season = unmask(Fragment, seasonRaw);
  const imageAsset = season.properties.posterImage ?? season.properties.thumbnailImage;
  const path = getPathForNode(season);
  const detail =
    season.episodeCount > 0
      ? `${season.episodeCount} ${season.episodeCount === 1 ? "episode" : "episodes"}`
      : formatReleaseYear(season.properties.firstAired, season.properties.lastAired ?? null);

  return (
    <div className="flex flex-col gap-2 overflow-hidden w-38">
      <PlayWrapper
        itemId={season.currentPlayable?.id}
        path={path}
        unavailable={season.unavailableAt != null}
        watchProgress={season.currentPlayable?.watchProgress}
      >
        <Image type={ImageType.Poster} asset={imageAsset} alt={season.properties.displayName} className="w-full" />
        <UnplayedItemsTab>{season.unplayedCount}</UnplayedItemsTab>
      </PlayWrapper>
      <Link to={path} className="block w-full truncate text-sm group">
        <span className="group-hover:underline">
          {season.properties.displayName || `Season ${season.properties.seasonNumber}`}
        </span>
        {detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
      </Link>
    </div>
  );
};
