import type { FC } from "react";
import { getPathForMedia, GetPathForMediaFrag } from "../lib/getPathForMedia";
import { Poster } from "./poster";
import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { PlayWrapper, PlayWrapperFrag } from "./play-wrapper";
import { cn } from "../lib/utils";

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
				<Poster imageUrl={media.posterUrl} alt={media.name} className="w-full" />
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
