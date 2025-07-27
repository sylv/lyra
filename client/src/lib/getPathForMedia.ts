import { graphql, readFragment, type FragmentOf } from "gql.tada";

export const GetPathForMediaFrag = graphql(`
	fragment GetPathForMedia on Media {
		id
		mediaType
		parentId
		seasonNumber
		episodeNumber
	}
`);

export const getPathForMedia = (mediaRaw: FragmentOf<typeof GetPathForMediaFrag>) => {
	const media = readFragment(GetPathForMediaFrag, mediaRaw);
	switch (media.mediaType) {
		case "SHOW":
			return `/series/${media.id}`;
		case "MOVIE":
			return `/movie/${media.id}`;
		case "EPISODE":
			// if (media.seasonNumber == null || media.episodeNumber == null) {
			// 	return `/series/${media.parentId}`;
			// }

			// return `/series/${media.parentId}/season/${media.seasonNumber}/episode/${media.episodeNumber}`;
			return `/series/${media.parentId}`;
	}
};
