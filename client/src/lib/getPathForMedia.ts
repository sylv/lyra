import { graphql, readFragment, type FragmentOf } from "gql.tada";

export const GetPathForMediaFrag = graphql(`
	fragment GetPathForMedia on Media {
		id
		mediaType
		parentId
		seasonNumber
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
			// todo: include episode number and highlight it on the page
			return `/series/${media.parentId}?seasons=${media.seasonNumber}`;
	}
};
