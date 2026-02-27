import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { graphql, type VariablesOf } from "gql.tada";
import { Fragment, useState } from "react";
import { MediaFilterList } from "@/components/media-filter-list";
import { MediaList, MediaListFrag } from "@/components/media-list";

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
	[MediaListFrag],
);

type RootNodeFilter = VariablesOf<typeof Query>["filter"];

export const Route = createFileRoute("/library/$libraryId")({
	component: LibraryRoute,
});

function LibraryRoute() {
	const { libraryId: rawLibraryId } = Route.useParams();
	const parsedLibraryId = Number(rawLibraryId);
	const libraryId = Number.isNaN(parsedLibraryId) ? null : parsedLibraryId;
	const [filter, setFilter] = useState<RootNodeFilter>({
		orderBy: "ADDED_AT",
	})

	const { data, loading, fetchMore } = useQuery(Query, {
		variables: {
			filter: {
				libraryId,
				...filter,
			},
		},
		skip: libraryId == null,
	})

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
						})
					}}
				/>
			</div>
		</Fragment>
	)
}
