import { MediaFilterList } from "@/components/media-filter-list";
import { MediaList } from "@/components/media-list";
import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { Fragment, useState } from "react";
import { graphql } from "../@generated/gql";
import { OrderBy, type RootNodeFilter } from "../@generated/gql/graphql";
import { client } from "../client";

const Query = graphql(
	`
	query GetLibraryMedia($filter: RootNodeFilter!, $after: String) {
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
	}
`,
);

export const Route = createFileRoute("/library/$libraryId")({
	component: LibraryRoute,
	loader: ({ params }) => {
		client.query({
			query: Query,
			variables: {
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
	const parsedLibraryId = Number(rawLibraryId);
	const libraryId = Number.isNaN(parsedLibraryId) ? null : parsedLibraryId;
	const [filter, setFilter] = useState<RootNodeFilter>({
		orderBy: OrderBy.Alphabetical,
	});

	const { data, loading, fetchMore } = useQuery(Query, {
		variables: {
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
