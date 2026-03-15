import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { GetPathForNodeFragment } from "../@generated/gql/graphql";

const NodeFragment = graphql(`
	fragment GetPathForNode on Node {
		id
		libraryId
	}
`);

export const getPathForNodeData = (node: GetPathForNodeFragment) => {
	return `/library/${node.libraryId}/node/${node.id}`;
};

export const getPathForNode = (nodeRaw: FragmentType<typeof NodeFragment>) => {
	const node = unmask(NodeFragment, nodeRaw);
	return getPathForNodeData(node);
};
