import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { graphql } from "../@generated/gql";
import { OrderBy, type RootNodeFilter } from "../@generated/gql/graphql";
import { client } from "../client";
import { MediaFilterList } from "../components/media-filter-list";
import { MediaList } from "../components/media-list";

const Query = graphql(
	`
	query GetAllMedia($filter: RootNodeFilter!, $after: String) {
		rootList(filter: $filter, first: 45, after: $after) {
			edges {
				node {
					...MediaList
				}
			}
			pageInfo {
				endCursor
				hasNextPage
			}
		}
	}
`,
);

export const Route = createFileRoute("/")({
	component: HomeRoute,
	loader: () => {
		client.query({
			query: Query,
			variables: {
				filter: {
					orderBy: OrderBy.AddedAt,
				},
			},
		});
	},
});

function HomeRoute() {
	const [filter, setFilter] = useState<RootNodeFilter>({
		orderBy: OrderBy.AddedAt,
	});

	const { data, loading, fetchMore } = useQuery(Query, {
		variables: {
			filter,
		},
	});

	return (
		<div>
			<div className="my-4 flex flex-col gap-2">
				<div className="flex flex-wrap gap-2">
					<MediaFilterList value={filter} onChange={(newFilter) => setFilter({ ...filter, ...newFilter })} />
				</div>
			</div>
			<div className="flex flex-wrap gap-4">
				<MediaList
					media={data?.rootList?.edges?.map((edge) => edge.node)}
					loading={loading}
					onLoadMore={() => {
						if (!data?.rootList?.pageInfo?.hasNextPage) return;
						fetchMore({
							variables: {
								after: data?.rootList?.pageInfo?.endCursor,
							},
						});
					}}
				/>
			</div>
		</div>
	);
}
