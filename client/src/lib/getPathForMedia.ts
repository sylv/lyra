import { graphql, readFragment, type FragmentOf } from "gql.tada";

export const GetPathForMediaFrag = graphql(`
	fragment GetPathForMedia on Node {
		id
		kind
		rootId
		parentId
		seasonNumber
	}
`);

export const getPathForMedia = (mediaRaw: FragmentOf<typeof GetPathForMediaFrag>) => {
	const media = readFragment(GetPathForMediaFrag, mediaRaw);
	switch (media.kind) {
		case "SERIES":
			return `/series/${media.id}`;
		case "MOVIE":
			return `/movie/${media.id}`;
		case "SEASON":
			return `/series/${media.rootId ?? media.id}?seasons=${media.seasonNumber ?? 1}`;
		case "EPISODE":
			// todo: include episode number and highlight it on the page
			return `/series/${media.rootId ?? media.parentId}?seasons=${media.seasonNumber ?? 1}`;
	}
};
