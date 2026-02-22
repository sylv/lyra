import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type { FC } from "react";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";

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
		<div className="flex flex-col gap-2 overflow-hidden w-38">
			<PlayWrapper itemId={season.playableItem?.id} path={path} watchProgress={season.watchProgress}>
				<Image type={ImageType.Poster} imageUrl={imageUrl} alt={season.name} className="w-full" />
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
