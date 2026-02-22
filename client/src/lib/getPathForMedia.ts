import { graphql, readFragment, type FragmentOf } from "gql.tada";

export const GetPathForRootFrag = graphql(`
	fragment GetPathForRoot on RootNode {
		id
		libraryId
	}
`);

export const GetPathForItemFrag = graphql(`
	fragment GetPathForItem on ItemNode {
		kind
		rootId
		seasonId
		parent {
			libraryId
		}
	}
`);

interface ItemPathData {
	kind: "MOVIE" | "EPISODE";
	rootId: string;
	seasonId: string | null;
	parent: {
		libraryId: number;
	} | null;
}

export const getPathForRoot = (mediaRaw: FragmentOf<typeof GetPathForRootFrag>) => {
	const media = readFragment(GetPathForRootFrag, mediaRaw);
	return `/library/${media.libraryId}/root/${media.id}`;
};

export const getPathForItemData = (media: ItemPathData) => {
	const libraryId = media.parent?.libraryId;

	if (!libraryId) {
		return "/";
	}

	switch (media.kind) {
		case "MOVIE":
			return `/library/${libraryId}/root/${media.rootId}`;
		case "EPISODE":
			if (!media.seasonId) {
				return `/library/${libraryId}/root/${media.rootId}`;
			}
			return `/library/${libraryId}/root/${media.rootId}/season/${media.seasonId}`;
	}
};

export const getPathForItem = (mediaRaw: FragmentOf<typeof GetPathForItemFrag>) => {
	const media = readFragment(GetPathForItemFrag, mediaRaw);
	return getPathForItemData(media);
};
