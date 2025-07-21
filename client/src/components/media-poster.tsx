import type { FC } from "react";
import { getPathForMedia, GetPathForMediaFrag } from "../lib/getPathForMedia";
import { Poster } from "./poster";
import { graphql, readFragment, type FragmentOf } from "gql.tada";

interface MediaPosterProps {
	media: FragmentOf<typeof MediaPosterFrag>;
}

export const MediaPosterFrag = graphql(
	`
	fragment MediaPoster on Media {
		id
		name
		posterUrl
		...GetPathForMedia
	}
`,
	[GetPathForMediaFrag],
);

export const MediaPoster: FC<MediaPosterProps> = ({ media: mediaRaw }) => {
	const media = readFragment(MediaPosterFrag, mediaRaw);
	const path = getPathForMedia(media);

	return (
		<a key={media.id} href={path}>
			<Poster imageUrl={media.posterUrl} alt={media.name} />
		</a>
	);
};
