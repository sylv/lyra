import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { ArrowDownNarrowWide, ChevronDown } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import type { MediaFilter, MediaType } from "../../@generated/enums.js";
import { FilterButton } from "../../components/filter-button.jsx";
import { MediaList, MediaListFrag } from "../../components/media-list.jsx";
import { useQueryState } from "../../hooks/use-query-state.js";

const Query = graphql(
	`
	query GetAllMedia($filter: MediaFilter!) {
		mediaList(filter: $filter) {
			id
			...MediaList
		}
	}
`,
	[MediaListFrag],
);

export default function Page() {
	const [search, setSearch] = useQueryState("search", "");
	const [selectedMediaTypes, setSelectedMediaTypes] = useQueryState<MediaType[]>("mediaTypes", []);
	const [debouncedSearch, setDebouncedSearch] = useState(search);

	// debounce search input
	useEffect(() => {
		const timer = setTimeout(() => {
			setDebouncedSearch(search);
		}, 300);

		return () => clearTimeout(timer);
	}, [search]);

	// prepare the query parameters
	const filter: MediaFilter = useMemo(
		() => ({
			parentId: null,
			search: debouncedSearch.trim() || null,
			mediaTypes: selectedMediaTypes.length > 0 ? selectedMediaTypes : null,
		}),
		[debouncedSearch, selectedMediaTypes],
	);

	const { data } = useQuery(Query, {
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
					value={search}
					onChange={(e) => setSearch(e.target.value)}
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
			<div className="m-4 flex flex-wrap gap-4">{data && <MediaList media={data.mediaList} />}</div>
		</div>
	);
}
