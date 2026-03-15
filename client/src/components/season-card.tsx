import { Link } from "@tanstack/react-router";
import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { SeasonCardFragment } from "../@generated/gql/graphql";
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
		name
		properties {
			seasonNumber
			posterImage {
				...ImageAsset
			}
			thumbnailImage {
				...ImageAsset
			}
			releasedAt
			endedAt
		}
		nextPlayable {
			id
			watchProgress {
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
			: formatReleaseYear(season.properties.releasedAt, season.properties.endedAt ?? null);

	return (
		<div className="flex flex-col gap-2 overflow-hidden w-38">
			<PlayWrapper itemId={season.nextPlayable?.id} path={path} watchProgress={season.nextPlayable?.watchProgress}>
				<Image type={ImageType.Poster} asset={imageAsset} alt={season.name} className="w-full" />
				<UnplayedItemsTab>{season.unplayedCount}</UnplayedItemsTab>
			</PlayWrapper>
			<Link to={path as never} className="block w-full truncate text-sm group">
				<span className="group-hover:underline">{season.name || `Season ${season.properties.seasonNumber}`}</span>
				{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
			</Link>
		</div>
	);
};
