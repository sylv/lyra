import { MediaFilterList } from "@/components/media-filter-list";
import { MediaList } from "@/components/media-list";
import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { Fragment, useState } from "react";
import { graphql } from "../@generated/gql";
import { NodeKind, OrderBy, type NodeFilter } from "../@generated/gql/graphql";
import { getApolloClient } from "../client";
import { useTitle } from "../hooks/use-title";

const Query = graphql(`
	query GetLibraryMedia($libraryId: Int!, $filter: NodeFilter!, $after: String) {
		nodeList(filter: $filter, first: 45, after: $after) {
			edges {
				node {
					id
					...MediaList
				}
			}
			pageInfo {
				endCursor
				hasNextPage
			}
		}
		library(libraryId: $libraryId) {
			id
			name
		}
	}
`);

export const Route = createFileRoute("/library/$libraryId")({
	component: LibraryRoute,
	loader: ({ params }) => {
		getApolloClient().query({
			query: Query,
			variables: {
				libraryId: Number(params.libraryId),
				filter: {
					libraryId: Number(params.libraryId),
					kinds: [NodeKind.Movie, NodeKind.Series],
					orderBy: OrderBy.Alphabetical,
				},
			},
		});
	},
});

function LibraryRoute() {
	const { libraryId: rawLibraryId } = Route.useParams();
	const libraryId = +rawLibraryId;
	const [filter, setFilter] = useState<NodeFilter>({
		kinds: [NodeKind.Movie, NodeKind.Series],
		orderBy: OrderBy.Alphabetical,
	});

	const { data, loading, fetchMore } = useQuery(Query, {
		variables: { libraryId, filter: { libraryId, ...filter } },
		skip: libraryId == null,
	});

	useTitle(data?.library.name);

	return (
		<Fragment>
			<div className="my-4 flex flex-col gap-2">
				<div className="flex flex-wrap gap-2">
					<MediaFilterList value={{ libraryId, ...filter }} onChange={setFilter} />
				</div>
			</div>
			<div className="flex flex-wrap gap-4">
				<MediaList
					media={data?.nodeList?.edges.map((edge) => edge?.node).filter((node) => node != null) ?? []}
					loading={loading}
					onLoadMore={() => {
						if (!data?.nodeList?.pageInfo?.hasNextPage) return;
						fetchMore({ variables: { after: data.nodeList.pageInfo.endCursor } });
					}}
				/>
			</div>
		</Fragment>
	);
}
