import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type React from "react";
import type { FC } from "react";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForRoot, GetPathForRootFrag } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { Image, ImageAssetFrag, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface MediaPosterProps {
	media: FragmentOf<typeof MediaPosterFrag>;
	className?: string;
	style?: React.CSSProperties;
}

export const MediaPosterFrag = graphql(
	`
	fragment MediaPoster on RootNode {
		id
		name
		kind
		properties {
			posterImage {
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
		seasonCount
		episodeCount
		...GetPathForRoot
	}
`,
	[GetPathForRootFrag, ImageAssetFrag],
);

export const MediaPoster: FC<MediaPosterProps> = ({ media: mediaRaw, className, style }) => {
	const media = readFragment(MediaPosterFrag, mediaRaw);
	const path = getPathForRoot(media);
	const detail = getRootPosterDetail(media);

	return (
		<div className={cn("flex flex-col gap-2 overflow-hidden select-none", className)} style={style}>
			<PlayWrapper itemId={media.playableItem?.id} path={path} watchProgress={media.watchProgress}>
				<Image type={ImageType.Poster} asset={media.properties.posterImage} alt={media.name} className="w-full" />
				<UnplayedItemsTab count={media.unplayedItems} />
			</PlayWrapper>
			<a href={path} className="block w-full truncate text-sm group">
				<span className="group-hover:underline">{media.name}</span>
				{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
			</a>
		</div>
	);
};

const getRootPosterDetail = (media: FragmentOf<typeof MediaPosterFrag>): string | number | null => {
	if (media.kind === "SERIES") {
		if (media.seasonCount > 0) {
			return formatCountLabel(media.seasonCount, "season", "seasons");
		}

		if (media.episodeCount > 0) {
			return formatCountLabel(media.episodeCount, "episode", "episodes");
		}
	}

	if (!media.properties.releasedAt) {
		return null;
	}

	return formatReleaseYear(media.properties.releasedAt, media.properties.endedAt ?? null) ?? null;
};

const formatCountLabel = (count: number, singular: string, plural: string): string => {
	return `${count} ${count === 1 ? singular : plural}`;
};
