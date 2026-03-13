import { Link } from "@tanstack/react-router";
import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { SeasonCardFragment } from "../@generated/gql/graphql";
import { formatReleaseYear } from "../lib/format-release-year";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface SeasonCardProps {
	season: FragmentType<typeof Fragment>;
	path: string;
}

const Fragment = graphql(
	`
	fragment SeasonCard on SeasonNode {
		id
		name
		seasonNumber
		order
		properties {
			posterImage {
				...ImageAsset
			}
			thumbnailImage {
				...ImageAsset
			}
			releasedAt
			endedAt
		}
		nextItem {
			id
			watchProgress {
				progressPercent
				completed
				updatedAt
			}
		}
		unplayedItems
		episodeCount
	}
`,
);

export const SeasonCard: FC<SeasonCardProps> = ({ season: seasonRaw, path }) => {
	const season = unmask(Fragment, seasonRaw);
	const imageAsset = season.properties.posterImage ?? season.properties.thumbnailImage;
	const detail = getSeasonPosterDetail(season);

	return (
		<div className="flex flex-col gap-2 overflow-hidden w-38">
			<PlayWrapper itemId={season.nextItem?.id} path={path} watchProgress={season.nextItem?.watchProgress}>
				<Image type={ImageType.Poster} asset={imageAsset} alt={season.name} className="w-full" />
				<UnplayedItemsTab count={season.unplayedItems} />
			</PlayWrapper>
			<Link to={path as never} className="block w-full truncate text-sm group">
				<span className="group-hover:underline">{season.name || `Season ${season.seasonNumber}`}</span>
				{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
			</Link>
		</div>
	);
};

const getSeasonPosterDetail = (season: SeasonCardFragment): string | number | null => {
	if (season.episodeCount > 0) {
		return formatCountLabel(season.episodeCount, "episode", "episodes");
	}

	if (!season.properties.releasedAt) {
		return null;
	}

	return formatReleaseYear(season.properties.releasedAt, season.properties.endedAt ?? null) ?? null;
};

const formatCountLabel = (count: number, singular: string, plural: string): string => {
	return `${count} ${count === 1 ? singular : plural}`;
};
