import { useQuery } from "@apollo/client/react";
import { graphql, type VariablesOf } from "gql.tada";
import { useState } from "react";
import { FilterButton } from "../../components/filter-button.jsx";
import { MediaFilterList } from "../../components/media-filter-list.jsx";
import { MediaList, MediaListFrag } from "../../components/media-list.jsx";

const Query = graphql(
	`
	query GetAllMedia($filter: NodeFilter!, $after: String) {
		nodeList(filter: $filter, first: 45, after: $after) {
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
	[MediaListFrag],
);

type NodeFilter = VariablesOf<typeof Query>["filter"];
type NodeKind = NonNullable<NonNullable<NodeFilter["kinds"]>[number]>;

export default function Page() {
	const [filter, setFilter] = useState<NodeFilter>({
		parentId: null,
		kinds: [],
		orderBy: "ADDED_AT",
	});

	const useFilterTypes = filter.kinds && filter.kinds.length > 0;
	const { data, loading, fetchMore } = useQuery(Query, {
		variables: {
			filter: {
				...filter,
				kinds: useFilterTypes ? filter.kinds : ["MOVIE", "SERIES"],
			},
		},
	});

	const handleMediaKindToggle = (kind: NodeKind) => {
		if (!filter.kinds) {
			setFilter({ ...filter, kinds: ["MOVIE", "SERIES"] });
			return;
		}

		const nextMediaKinds = filter.kinds.includes(kind)
			? filter.kinds.filter((type) => type !== kind)
			: [...filter.kinds, kind];

		setFilter({ ...filter, kinds: nextMediaKinds });
	};

	return (
		<div>
			<div className="my-4 flex flex-col gap-2">
				<div className="flex flex-wrap gap-2">
					<FilterButton onClick={() => handleMediaKindToggle("SERIES")} active={filter.kinds?.includes("SERIES")}>
						Series
					</FilterButton>
					<FilterButton onClick={() => handleMediaKindToggle("MOVIE")} active={filter.kinds?.includes("MOVIE")}>
						Movies
					</FilterButton>
					<MediaFilterList value={filter} onChange={(newFilter) => setFilter({ ...filter, ...newFilter })} />
				</div>
			</div>
			<div className="flex flex-wrap gap-4">
				<MediaList
					media={data?.nodeList?.edges?.map((edge) => edge.node)}
					loading={loading}
					onLoadMore={() => {
						if (!data?.nodeList?.pageInfo?.hasNextPage) return;
						fetchMore({
							variables: {
								after: data?.nodeList?.pageInfo?.endCursor,
							},
						});
					}}
				/>
			</div>
		</div>
	);
}
