import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { getPathForCollection } from "../lib/getPathForMedia";
import { NodePosterDetail } from "./nodes/node-poster-detail";
import type { FC } from "react";

export const CollectionShelfFragment = graphql(`
	fragment CollectionShelf on Collection {
		id
		name
		nodeList(first: 12) {
			nodes {
				id
				...NodePoster
			}
		}
	}
`);

export const CollectionShelf: FC<{
	collection: FragmentType<typeof CollectionShelfFragment>;
}> = ({ collection: collectionRaw }) => {
	const collection = unmask(CollectionShelfFragment, collectionRaw);
	if (collection.nodeList.nodes.length === 0) return null;

	return (
		<section className="space-y-4">
			<div className="flex items-end justify-between gap-4">
				<div>
					<Link to={getPathForCollection(collection.id)} className="text-xl font-semibold hover:underline">
						{collection.name}
					</Link>
				</div>
			</div>
			<div className="-mx-6 overflow-x-auto px-6">
				<div className="flex gap-4">
					{collection.nodeList.nodes.map((node) => (
						<div className="w-42" key={node.id}>
							<NodePosterDetail node={node} />
						</div>
					))}
				</div>
			</div>
		</section>
	);
};
