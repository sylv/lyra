import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type { FC } from "react";
import { Image, ImageAssetFrag, ImageType } from "./image";
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
			posterImage {
				...ImageAsset
			}
			thumbnailImage {
				...ImageAsset
			}
		}
		playableItem {
			id
		}
		watchProgress {
			progressPercent
			updatedAt
		}
	}
`, [ImageAssetFrag]);

export const SeasonCard: FC<SeasonCardProps> = ({ season: seasonRaw, path }) => {
	const season = readFragment(SeasonCardFrag, seasonRaw);
	const imageAsset = season.properties.posterImage ?? season.properties.thumbnailImage;

	return (
		<div className="flex flex-col gap-2 overflow-hidden w-38">
			<PlayWrapper itemId={season.playableItem?.id} path={path} watchProgress={season.watchProgress}>
				<Image type={ImageType.Poster} asset={imageAsset} alt={season.name} className="w-full" />
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
