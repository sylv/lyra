import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type { FC } from "react";
import { PlayWrapper } from "./play-wrapper";
import { Poster } from "./poster";
import { Skeleton } from "./skeleton";

interface SeasonCardProps {
	season: FragmentOf<typeof SeasonCardFrag>;
	path: string;
}

export const SeasonCardFrag = graphql(`
	fragment SeasonCard on SeasonNode {
		id
		name
		seasonNumber
		order
		properties {
			posterUrl
			thumbnailUrl
		}
		playableItem {
			id
		}
		watchProgress {
			progressPercent
			updatedAt
		}
	}
`);

export const SeasonCard: FC<SeasonCardProps> = ({ season: seasonRaw, path }) => {
	const season = readFragment(SeasonCardFrag, seasonRaw);
	const imageUrl = season.properties.posterUrl ?? season.properties.thumbnailUrl;

	return (
		<div className="flex flex-col gap-2 overflow-hidden">
			<PlayWrapper itemId={season.playableItem?.id} path={path} watchProgress={season.watchProgress}>
				<Poster imageUrl={imageUrl} alt={season.name} className="w-full" />
			</PlayWrapper>
			<a
				href={path}
				className="block w-full truncate text-sm font-semibold text-zinc-400 transition-colors hover:text-zinc-300 hover:underline"
			>
				{season.name || `Season ${season.seasonNumber}`}
			</a>
		</div>
	);
};

export const SeasonCardSkeleton: FC = () => {
	return (
		<div className="flex flex-col gap-2 overflow-hidden">
			<Skeleton className="aspect-[2/3] w-full rounded-md" />
			<Skeleton className="h-4 w-3/4" />
		</div>
	);
};
