import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { useState } from "react";
import type { MediaFilter, MediaKind } from "../../@generated/enums.js";
import { FilterButton } from "../../components/filter-button.jsx";
import { MediaFilterList } from "../../components/media-filter-list.jsx";
import { MediaList, MediaListFrag } from "../../components/media-list.jsx";
import { setIsSearchOpen } from "../../components/search/search-modal.jsx";

const Query = graphql(
	`
	query GetAllMedia($filter: MediaFilter!, $after: String) {
		mediaList(filter: $filter, first: 45, after: $after) {
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

export default function Page() {
	const [filter, setFilter] = useState<MediaFilter>({
		parentId: null,
		kinds: [],
		orderBy: "ADDED_AT",
	});

	const useFilterTypes = filter.kinds && filter.kinds.length > 0;
	const { data, loading, fetchMore } = useQuery(Query, {
		variables: {
			filter: {
				...filter,
				kinds: useFilterTypes ? filter.kinds : ["MOVIE", "SHOW"],
			},
		},
	});

	const handleMediaKindToggle = (kind: MediaKind) => {
		if (!filter.kinds) {
			setFilter({ ...filter, kinds: ["MOVIE", "SHOW"] });
			return;
		}

		const nextMediaKinds = filter.kinds.includes(kind)
			? filter.kinds.filter((type) => type !== kind)
			: [...filter.kinds, kind];

		setFilter({ ...filter, kinds: nextMediaKinds });
	};

	return (
		<div className="container mx-auto">
			<div className="m-4 flex flex-col gap-2">
				<input
					type="text"
					placeholder="Search"
					className="border border-zinc-700/50 text-zinc-200 rounded-lg px-4 py-2 text-sm max-w-sm outline-none focus:bg-zinc-400/15 hover:bg-zinc-400/10 transition-colors"
					onFocus={() => setIsSearchOpen(true)}
				/>
				<div className="flex flex-wrap gap-2">
					<FilterButton onClick={() => handleMediaKindToggle("SHOW")} active={filter.kinds?.includes("SHOW")}>
						Series
					</FilterButton>
					<FilterButton onClick={() => handleMediaKindToggle("MOVIE")} active={filter.kinds?.includes("MOVIE")}>
						Movies
					</FilterButton>
					<MediaFilterList value={filter} onChange={(newFilter) => setFilter({ ...filter, ...newFilter })} />
				</div>
			</div>
			<div className="m-4 flex flex-wrap gap-4">
				<MediaList
					media={data?.mediaList?.edges?.map((edge) => edge.node)}
					loading={loading}
					onLoadMore={() => {
						if (!data?.mediaList?.pageInfo?.hasNextPage) return;
						fetchMore({
							variables: {
								after: data?.mediaList?.pageInfo?.endCursor,
							},
						});
					}}
				/>
			</div>
		</div>
	);
}
