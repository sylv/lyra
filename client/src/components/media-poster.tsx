import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type React from "react";
import type { FC } from "react";
import { getPathForMedia, GetPathForMediaFrag } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { PlayWrapper, PlayWrapperFrag } from "./play-wrapper";
import { Poster } from "./poster";
import { Thumbnail } from "./thumbnail";

interface MediaPosterProps {
	media: FragmentOf<typeof MediaPosterFrag>;
	className?: string;
	style?: React.CSSProperties;
}

export const MediaPosterFrag = graphql(
	`
	fragment MediaPoster on Media {
		id
		name
		posterUrl
		mediaType
		thumbnailUrl
		...GetPathForMedia
		...PlayWrapper
	}
`,
	[GetPathForMediaFrag, PlayWrapperFrag],
);

export const MediaPoster: FC<MediaPosterProps> = ({ media: mediaRaw, className, style }) => {
	const media = readFragment(MediaPosterFrag, mediaRaw);
	const path = getPathForMedia(media);

	return (
		<div className={cn("flex flex-col gap-2 overflow-hidden truncate", className)} style={style}>
			<PlayWrapper media={media}>
				{media.mediaType === "EPISODE" ? (
					<Thumbnail imageUrl={media.thumbnailUrl} alt={media.name} className="w-full" />
				) : (
					<Poster imageUrl={media.posterUrl} alt={media.name} className="w-full" />
				)}
			</PlayWrapper>
			<a
				href={path}
				className="text-sm font-semibold text-zinc-400 hover:underline hover:text-zinc-300 transition-colors"
			>
				{media.name}
			</a>
		</div>
	);
};
