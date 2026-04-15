import { graphql, unmask, type FragmentType } from "../@generated/gql";

const NodeFragment = graphql(`
  fragment GetPathForNode on Node {
    id
    libraryId
  }
`);

export const getPathForNode = (nodeRaw: FragmentType<typeof NodeFragment>) => {
  const node = unmask(NodeFragment, nodeRaw);
  return `/library/${node.libraryId}/node/${node.id}`;
};

export const getPathForCollection = (collectionId: string) => `/collection/${collectionId}`;
