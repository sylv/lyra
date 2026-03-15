import { Link } from "@tanstack/react-router";
import type React from "react";
import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { MediaPosterFragment } from "../@generated/gql/graphql";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForNode } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface MediaPosterProps {
	media: FragmentType<typeof Fragment>;
	className?: string;
	style?: React.CSSProperties;
}

const Fragment = graphql(`
	fragment MediaPoster on Node {
		id
		name
		kind
		libraryId
		properties {
			posterImage {
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
		seasonCount
		episodeCount
		...GetPathForNode
	}
`);

export const MediaPoster: FC<MediaPosterProps> = ({ media: mediaRaw, className, style }) => {
	const media = unmask(Fragment, mediaRaw);
	const path = getPathForNode(media);
	const detail = getPosterDetail(media);

	return (
		<div className={cn("flex flex-col gap-2 overflow-hidden select-none", className)} style={style}>
			<PlayWrapper
				itemId={media.nextPlayable?.id ?? media.id}
				path={path}
				watchProgress={media.nextPlayable?.watchProgress ?? null}
			>
				<Image type={ImageType.Poster} asset={media.properties.posterImage} alt={media.name} className="w-full" />
				<UnplayedItemsTab>{media.unplayedCount}</UnplayedItemsTab>
			</PlayWrapper>
			<Link to={path as never} className="block w-full truncate text-sm group">
				<span className="group-hover:underline">{media.name}</span>
				{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
			</Link>
		</div>
	);
};

const getPosterDetail = (media: MediaPosterFragment): string | number | null => {
	if (media.kind === "SERIES") {
		if (media.seasonCount > 0) return `${media.seasonCount} ${media.seasonCount === 1 ? "season" : "seasons"}`;
		if (media.episodeCount > 0) return `${media.episodeCount} ${media.episodeCount === 1 ? "episode" : "episodes"}`;
	}

	if (!media.properties.releasedAt) return null;
	return formatReleaseYear(media.properties.releasedAt, media.properties.endedAt ?? null) ?? null;
};
