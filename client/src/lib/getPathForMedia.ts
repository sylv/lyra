import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { GetPathForItemFragment } from "../@generated/gql/graphql";

const RootFragment = graphql(`
	fragment GetPathForRoot on RootNode {
		id
		libraryId
	}
`);

const ItemFragment = graphql(`
	fragment GetPathForItem on ItemNode {
		kind
		rootId
		seasonId
		parent {
			libraryId
		}
	}
`);

export const getPathForRoot = (mediaRaw: FragmentType<typeof RootFragment>) => {
	const media = unmask(RootFragment, mediaRaw);
	return `/library/${media.libraryId}/${media.id}`;
};

export const getPathForItemData = (media: GetPathForItemFragment) => {
	const libraryId = media.parent?.libraryId;

	if (!libraryId) {
		return "/";
	}

	switch (media.kind) {
		case "MOVIE":
			return `/library/${libraryId}/${media.rootId}`;
		case "EPISODE":
			if (!media.seasonId) {
				return `/library/${libraryId}/${media.rootId}`;
			}
			return `/library/${libraryId}/${media.rootId}/${media.seasonId}`;
		default:
			throw new Error(`Unknown media kind: ${media.kind}`);
	}
};

export const getPathForItem = (mediaRaw: FragmentType<typeof ItemFragment>) => {
	const media = unmask(ItemFragment, mediaRaw);
	return getPathForItemData(media);
};
