import { ArrowRightIcon } from "lucide-react";
import { type FC } from "react";
import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { getPathForCollection } from "../lib/getPathForMedia";
import { NodePosterDetail } from "./nodes/node-poster-detail";
import { ShelfCarousel } from "./shelf-carousel";

export const CollectionShelfFragment = graphql(`
  fragment CollectionShelf on Collection {
    id
    name
    itemCount
    nodeList(first: 12) {
      nodes {
        id
        ...NodePoster
      }
      pageInfo {
        hasNextPage
      }
    }
  }
`);

export const CollectionShelf: FC<{
  collection: FragmentType<typeof CollectionShelfFragment>;
}> = ({ collection: collectionRaw }) => {
  const collection = unmask(CollectionShelfFragment, collectionRaw);
  if (collection.nodeList.nodes.length === 0) return null;
  const collectionPath = getPathForCollection(collection.id);
  const hasMore = collection.nodeList.pageInfo.hasNextPage;

  return (
    <ShelfCarousel
      title={
        <Link to={collectionPath} className="truncate text-xl font-semibold hover:underline">
          {collection.name}
        </Link>
      }
    >
      {collection.nodeList.nodes.map((node) => (
        <div className="min-w-0 flex-[0_0_9rem]" key={node.id}>
          <NodePosterDetail node={node} />
        </div>
      ))}
      {hasMore ? (
        <div className="min-w-0 flex-[0_0_9rem]">
          <Link to={collectionPath} className="group flex h-full flex-col gap-2 outline-none">
            <div className="flex flex-col h-full justify-between rounded-sm border border-zinc-700/50 p-4 transition-colors">
              <div className="uppercase font-semibold text-[11px] text-zinc-500">{collection.name}</div>
              <div>
                <div className="text-sm font-semibold text-zinc-100 group-hover:underline">View More</div>
                <div className="mt-1 flex items-center gap-2 text-xs text-zinc-400">
                  <div>{collection.itemCount} items</div>
                  <ArrowRightIcon className="size-4 transition-transform group-hover:translate-x-0.5" />
                </div>
              </div>
            </div>
          </Link>
        </div>
      ) : null}
    </ShelfCarousel>
  );
};
