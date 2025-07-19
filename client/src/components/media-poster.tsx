import type { FC } from "react";
import type { MediaWithFirstConnection } from "../@generated/server";
import { getPathForMedia } from "../lib/getPathForMedia";
import { Poster } from "./poster";

interface MediaPosterProps {
	item: MediaWithFirstConnection;
}

export const MediaPoster: FC<MediaPosterProps> = ({ item }) => {
	const path = getPathForMedia(item.media);

	return (
		<a key={item.media.id} href={path}>
			<Poster imageUrl={item.media.poster_url} alt={item.media.name} />
		</a>
	);
};
