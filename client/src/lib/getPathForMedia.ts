import { graphql, readFragment, type FragmentOf } from "gql.tada";

export const GetPathForRootFrag = graphql(`
	fragment GetPathForRoot on RootNode {
		id
		kind
	}
`);

export const GetPathForItemFrag = graphql(`
	fragment GetPathForItem on ItemNode {
		id
		kind
		rootId
		seasonId
		properties {
			seasonNumber
		}
	}
`);

export const getPathForRoot = (mediaRaw: FragmentOf<typeof GetPathForRootFrag>) => {
	const media = readFragment(GetPathForRootFrag, mediaRaw);
	switch (media.kind) {
		case "SERIES":
			return `/series/${media.id}`;
		case "MOVIE":
			return `/movie/${media.id}`;
	}
};

export const getPathForItem = (mediaRaw: FragmentOf<typeof GetPathForItemFrag>) => {
	const media = readFragment(GetPathForItemFrag, mediaRaw);
	switch (media.kind) {
		case "MOVIE":
			return `/movie/${media.rootId}`;
		case "EPISODE":
			return `/series/${media.rootId}?seasons=${media.properties.seasonNumber ?? 1}`;
	}
};
