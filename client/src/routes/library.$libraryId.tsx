import { MediaFilterList } from "@/components/media-filter-list";
import { MediaList } from "@/components/media-list";
import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { Fragment, useState } from "react";
import { graphql } from "../@generated/gql";
import { OrderBy, type RootNodeFilter } from "../@generated/gql/graphql";
import { client } from "../client";
import { useTitle } from "../hooks/use-title";

const Query = graphql(`
	query GetLibraryMedia($libraryId: Int!, $filter: RootNodeFilter!, $after: String) {
		rootList(filter: $filter, first: 45, after: $after) {
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
		client.query({
			query: Query,
			variables: {
				libraryId: Number(params.libraryId),
				filter: {
					libraryId: Number(params.libraryId),
					orderBy: OrderBy.Alphabetical,
				},
			},
		});
	},
});

function LibraryRoute() {
	const { libraryId: rawLibraryId } = Route.useParams();
	const libraryId = +rawLibraryId;
	const [filter, setFilter] = useState<RootNodeFilter>({
		orderBy: OrderBy.Alphabetical,
	});

	const { data, loading, fetchMore } = useQuery(Query, {
		variables: {
			libraryId: libraryId,
			filter: {
				libraryId,
				...filter,
			},
		},
		skip: libraryId == null,
	});

	const media =
		libraryId == null
			? []
			: (data?.rootList?.edges
					.map((edge) => edge?.node)
					.filter((node): node is NonNullable<typeof node> => node != null) ?? []);

	useTitle(data?.library.name);

	return (
		<Fragment>
			<div className="my-4 flex flex-col gap-2">
				<div className="flex flex-wrap gap-2">
					<MediaFilterList value={filter} onChange={(newFilter) => setFilter({ ...filter, ...newFilter })} />
				</div>
			</div>
			<div className="flex flex-wrap gap-4">
				<MediaList
					media={media}
					loading={loading}
					onLoadMore={() => {
						if (libraryId == null) return;
						if (!data?.rootList?.pageInfo?.hasNextPage) return;
						fetchMore({
							variables: {
								after: data.rootList.pageInfo.endCursor,
							},
						});
					}}
				/>
			</div>
		</Fragment>
	);
}
