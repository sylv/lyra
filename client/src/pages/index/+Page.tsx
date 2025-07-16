import { ArrowDownNarrowWide, ChevronDown } from "lucide-react";
import { FilterButton } from "../../components/filter-button.jsx";
import { Poster } from "../../components/poster.jsx";
import { getPathForMedia } from "../../lib/getPathForMedia.js";
import { trpc } from "../trpc.js";
import { useState, useEffect, useMemo } from "react";
import type { GetAllMediaFilter, MediaType } from "../../@generated/server.js";

export default function Page() {
	const [search, setSearch] = useState("");
	const [debouncedSearch, setDebouncedSearch] = useState("");
	const [selectedMediaTypes, setSelectedMediaTypes] = useState<MediaType[]>([]);

	// debounce search input
	useEffect(() => {
		const timer = setTimeout(() => {
			setDebouncedSearch(search);
		}, 300);

		return () => clearTimeout(timer);
	}, [search]);

	// prepare the query parameters
	const filter: GetAllMediaFilter = useMemo(
		() => ({
			parent_id: null,
			search: debouncedSearch.trim() || null,
			media_types: selectedMediaTypes.length > 0 ? selectedMediaTypes : null,
		}),
		[debouncedSearch, selectedMediaTypes],
	);

	const [media] = trpc.get_all_media.useSuspenseQuery(
		{ filter },
		{
			refetchInterval: 5000,
		},
	);

	const handleMediaTypeToggle = (mediaType: MediaType) => {
		setSelectedMediaTypes((prev) =>
			prev.includes(mediaType)
				? prev.filter((type) => type !== mediaType)
				: [...prev, mediaType],
		);
	};

	return (
		<div className="container mx-auto">
			<div className="m-4 flex flex-col gap-2">
				<input
					type="text"
					placeholder="Search"
					className="bg-zinc-800 rounded-lg px-4 py-2 text-sm max-w-sm outline-none"
					value={search}
					onChange={(e) => setSearch(e.target.value)}
				/>
				<div className="flex flex-wrap gap-2">
					<FilterButton
						onClick={() => handleMediaTypeToggle("Show")}
						active={selectedMediaTypes.includes("Show")}
					>
						Series
					</FilterButton>
					<FilterButton
						onClick={() => handleMediaTypeToggle("Movie")}
						active={selectedMediaTypes.includes("Movie")}
					>
						Movies
					</FilterButton>
					<FilterButton onClick={() => {}}>
						<ArrowDownNarrowWide className="h-3.5 w-3.5 text-zinc-500" />
						Added <ChevronDown className="h-3 w-3" />
					</FilterButton>
				</div>
			</div>
			<div className="m-4 flex flex-wrap gap-4">
				{media.map(({ media, default_connection }) => {
					const path = getPathForMedia(media);
					return (
						<a key={media.id} href={path}>
							<Poster imageUrl={media.poster_url} alt={media.name} />
						</a>
					);
				})}
			</div>
		</div>
	);
}
