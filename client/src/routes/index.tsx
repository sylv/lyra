import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { graphql } from "../@generated/gql";
import { NodeKind, OrderBy, type NodeFilter } from "../@generated/gql/graphql";
import { getApolloClient } from "../client";
import { NodeFilterList } from "../components/nodes/node-filter-list";
import { NodeList } from "../components/nodes/node-list";
import { useTitle } from "../hooks/use-title";

const Query = graphql(`
	query GetAllNodes($filter: NodeFilter!, $after: String) {
		nodeList(filter: $filter, first: 45, after: $after) {
			edges {
				node {
					...NodeList
				}
			}
			pageInfo {
				endCursor
				hasNextPage
			}
		}
	}
`);

export const Route = createFileRoute("/")({
	component: HomeRoute,
	loader: () => {
		getApolloClient().query({
			query: Query,
			variables: {
				filter: {
					kinds: [NodeKind.Movie, NodeKind.Series],
					orderBy: OrderBy.AddedAt,
				},
			},
		});
	},
});

function HomeRoute() {
	const [filter, setFilter] = useState<NodeFilter>({
		kinds: [NodeKind.Movie, NodeKind.Series],
		orderBy: OrderBy.AddedAt,
	});
	const { data, loading, fetchMore } = useQuery(Query, { variables: { filter } });
	useTitle("Home");

	return (
		<div>
			<div className="my-4 flex flex-col gap-2">
				<div className="flex flex-wrap gap-2">
					<NodeFilterList value={filter} onChange={setFilter} />
				</div>
			</div>
			<div className="flex flex-wrap gap-4">
				<NodeList
					nodes={data?.nodeList?.edges?.map((edge) => edge.node)}
					loading={loading}
					onLoadMore={() => {
						if (!data?.nodeList?.pageInfo?.hasNextPage) return;
						fetchMore({ variables: { after: data.nodeList.pageInfo.endCursor } });
					}}
				/>
			</div>
		</div>
	);
}
