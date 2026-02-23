import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type { FC } from "react";
import { formatReleaseYear } from "../lib/format-release-year";
import { Image, ImageAssetFrag, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface SeasonCardProps {
	season: FragmentOf<typeof SeasonCardFrag>;
	path: string;
}

export const SeasonCardFrag = graphql(
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
		playableItem {
			id
		}
		watchProgress {
			progressPercent
			updatedAt
		}
		unplayedItems
		episodeCount
	}
`,
	[ImageAssetFrag],
);

export const SeasonCard: FC<SeasonCardProps> = ({ season: seasonRaw, path }) => {
	const season = readFragment(SeasonCardFrag, seasonRaw);
	const imageAsset = season.properties.posterImage ?? season.properties.thumbnailImage;
	const detail = getSeasonPosterDetail(season);

	return (
		<div className="flex flex-col gap-2 overflow-hidden w-38">
			<PlayWrapper itemId={season.playableItem?.id} path={path} watchProgress={season.watchProgress}>
				<Image type={ImageType.Poster} asset={imageAsset} alt={season.name} className="w-full" />
				<UnplayedItemsTab count={season.unplayedItems} />
			</PlayWrapper>
			<a href={path} className="block w-full truncate text-sm group">
				<span className="group-hover:underline">{season.name || `Season ${season.seasonNumber}`}</span>
				{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
			</a>
		</div>
	);
};

const getSeasonPosterDetail = (season: FragmentOf<typeof SeasonCardFrag>): string | number | null => {
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
