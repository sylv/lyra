import { useQuery } from "@apollo/client/react";
import { graphql, type VariablesOf } from "gql.tada";
import { useState } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { MediaFilterList } from "../../../components/media-filter-list";
import { MediaList, MediaListFrag } from "../../../components/media-list";

const Query = graphql(
	`
	query GetLibraryMedia($filter: RootNodeFilter!, $after: String) {
		libraries {
			id
			name
		}
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

export default function Page() {
	const pageContext = usePageContext();
	const rawLibraryId = Number(pageContext.routeParams.libraryId);
	const libraryId = Number.isNaN(rawLibraryId) ? null : rawLibraryId;
	const [filter, setFilter] = useState<RootNodeFilter>({
		orderBy: "ADDED_AT",
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
					?.map((edge) => edge?.node)
					.filter((node): node is NonNullable<typeof node> => node != null) ?? []);
	const libraryName = data?.libraries?.find((library) => library.id === libraryId)?.name ?? null;

	return (
		<div>
			<div className="my-4 flex flex-col gap-2">
				<h1 className="text-xl font-semibold text-zinc-200">{libraryName || "Library"}</h1>
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
		</div>
	);
}
