import { Trash2Icon } from "lucide-react";
import { useEffect, useState } from "react";
import { Navigate, useNavigate, useParams } from "react-router";
import { useMutation, useQuery } from "urql";
import { graphql } from "../@generated/gql";
import { Button, ButtonStyle } from "../components/button";
import { NodePosterDetail } from "../components/nodes/node-poster-detail";
import { ViewLoader } from "../components/view-loader";
import { useTitle } from "../hooks/use-title";

const CollectionQuery = graphql(`
	query CollectionPage($collectionId: String!, $after: String, $first: Int!) {
		collection(collectionId: $collectionId) {
			id
			name
			description
			itemCount
			canDelete
			nodeList(after: $after, first: $first) {
				nodes {
					id
					...NodePoster
				}
				pageInfo {
					endCursor
					hasNextPage
				}
			}
		}
	}
`);

const DeleteCollectionMutation = graphql(`
	mutation DeleteCollection($collectionId: String!) {
		deleteCollection(collectionId: $collectionId)
	}
`);

const PAGE_SIZE = 30;

export function CollectionRoute() {
	const { collectionId } = useParams<{ collectionId: string }>();
	const navigate = useNavigate();
	const [after, setAfter] = useState<string | null>(null);
	const [items, setItems] = useState<any[]>([]);
	const [{ data, fetching }] = useQuery({
		query: CollectionQuery,
		variables: { collectionId: collectionId!, after, first: PAGE_SIZE },
		pause: !collectionId,
		context: { suspense: after == null },
	});
	const [{ fetching: deleting }, deleteCollection] = useMutation(DeleteCollectionMutation);

	const collection = data?.collection ?? null;
	useTitle(collection?.name ?? "Collection");

	useEffect(() => {
		setAfter(null);
		setItems([]);
	}, [collectionId]);

	useEffect(() => {
		if (!collection) return;
		setItems((prev) => {
			if (after == null) return [...collection.nodeList.nodes];
			const byId = new Map(prev.map((node) => [node.id, node]));
			for (const node of collection.nodeList.nodes) {
				byId.set(node.id, node);
			}
			return [...byId.values()];
		});
	}, [after, collection]);

	if (!collectionId) return null;
	if (data && !collection) return <Navigate to="/collections" replace />;

	const handleDelete = async () => {
		const result = await deleteCollection({ collectionId });
		if (!result.error && result.data?.deleteCollection) {
			navigate("/collections");
		}
	};

	return (
		<div className="space-y-2 py-6">
			<div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
				<div>
					<h1 className="text-2xl font-semibold">{collection?.name}</h1>
				</div>
				{collection?.canDelete ? (
					<Button
						style={ButtonStyle.Transparent}
						icon={["delete-collection", Trash2Icon]}
						iconSide="left"
						loading={deleting}
						onClick={() => void handleDelete()}
					>
						Delete Collection
					</Button>
				) : null}
			</div>
			<div className="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-4">
				{items.map((node) => (
					<NodePosterDetail key={node.id} node={node} />
				))}
			</div>

			{collection?.nodeList.pageInfo.hasNextPage && collection.nodeList.pageInfo.endCursor ? (
				<>
					{fetching ? <div className="text-sm text-zinc-400">Loading more...</div> : null}
					<ViewLoader
						onLoadMore={() => {
							if (fetching) return;
							setAfter(collection.nodeList.pageInfo.endCursor ?? null);
						}}
					/>
				</>
			) : null}
		</div>
	);
}
