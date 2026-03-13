import { Link } from "@tanstack/react-router";
import type React from "react";
import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { MediaPosterFragment } from "../@generated/gql/graphql";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForRoot } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface MediaPosterProps {
	media: FragmentType<typeof Fragment>;
	className?: string;
	style?: React.CSSProperties;
}

const Fragment = graphql(
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
		nextItem {
			id
			watchProgress {
				progressPercent
				completed
				updatedAt
			}
		}
		unplayedItems
		seasonCount
		episodeCount
		...GetPathForRoot
	}
`,
);

export const MediaPoster: FC<MediaPosterProps> = ({ media: mediaRaw, className, style }) => {
	const media = unmask(Fragment, mediaRaw);
	const path = getPathForRoot(media);
	const detail = getRootPosterDetail(media);

	return (
		<div className={cn("flex flex-col gap-2 overflow-hidden select-none", className)} style={style}>
			<PlayWrapper itemId={media.nextItem?.id} path={path} watchProgress={media.nextItem?.watchProgress}>
				<Image type={ImageType.Poster} asset={media.properties.posterImage} alt={media.name} className="w-full" />
				<UnplayedItemsTab>{media.unplayedItems}</UnplayedItemsTab>
			</PlayWrapper>
			<Link to={path as never} className="block w-full truncate text-sm group">
				<span className="group-hover:underline">{media.name}</span>
				{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
			</Link>
		</div>
	);
};

const getRootPosterDetail = (media: MediaPosterFragment): string | number | null => {
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
