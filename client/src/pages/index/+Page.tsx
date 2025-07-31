import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { ArrowDownNarrowWide, ChevronDown } from "lucide-react";
import { useMemo } from "react";
import type { MediaFilter, MediaType } from "../../@generated/enums.js";
import { FilterButton } from "../../components/filter-button.jsx";
import { MediaList, MediaListFrag } from "../../components/media-list.jsx";
import { setIsSearchOpen } from "../../components/search/search-modal.jsx";
import { useQueryState } from "../../hooks/use-query-state.js";

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
			}
		}
	}
`,
	[MediaListFrag],
);

export default function Page() {
	const [selectedMediaTypes, setSelectedMediaTypes] = useQueryState<MediaType[]>("mediaTypes", ["MOVIE", "SHOW"]);
	// const [debouncedSearch, setDebouncedSearch] = useState(search);

	// // debounce search input
	// useEffect(() => {
	// 	const timer = setTimeout(() => {
	// 		setDebouncedSearch(search);
	// 	}, 300);

	// 	return () => clearTimeout(timer);
	// }, [search]);

	// prepare the query parameters
	const filter: MediaFilter = useMemo(
		() => ({
			parentId: null,
			// search: debouncedSearch.trim() || null,
			mediaTypes: selectedMediaTypes.length > 0 ? selectedMediaTypes : ["MOVIE", "SHOW"],
		}),
		[selectedMediaTypes],
	);

	const { data, loading, fetchMore } = useQuery(Query, {
		variables: { filter },
	});

	const handleMediaTypeToggle = (mediaType: MediaType) => {
		const nextMediaTypes = selectedMediaTypes.includes(mediaType)
			? selectedMediaTypes.filter((type) => type !== mediaType)
			: [...selectedMediaTypes, mediaType];

		setSelectedMediaTypes(nextMediaTypes);
	};

	return (
		<div className="container mx-auto">
			<div className="m-4 flex flex-col gap-2">
				<input
					type="text"
					placeholder="Search"
					className="border border-zinc-700/50 text-zinc-200 rounded-lg px-4 py-2 text-sm max-w-sm outline-none focus:bg-zinc-400/15 hover:bg-zinc-400/10 transition-colors"
					// value={search}
					// onChange={(e) => setSearch(e.target.value)}
					onFocus={() => setIsSearchOpen(true)}
				/>
				<div className="flex flex-wrap gap-2">
					<FilterButton onClick={() => handleMediaTypeToggle("SHOW")} active={selectedMediaTypes.includes("SHOW")}>
						Series
					</FilterButton>
					<FilterButton onClick={() => handleMediaTypeToggle("MOVIE")} active={selectedMediaTypes.includes("MOVIE")}>
						Movies
					</FilterButton>
					<FilterButton onClick={() => {}}>
						<ArrowDownNarrowWide className="h-3.5 w-3.5 text-zinc-500" />
						Added <ChevronDown className="h-3 w-3" />
					</FilterButton>
				</div>
			</div>
			<div className="m-4 flex flex-wrap gap-4">
				<MediaList
					media={data?.mediaList?.edges?.map((edge) => edge.node)}
					loading={loading}
					onLoadMore={() => {
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
