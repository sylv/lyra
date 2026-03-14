import { useQuery } from "@apollo/client/react";
import { Link } from "@tanstack/react-router";
import { SearchAlertIcon, SearchIcon, TriangleAlertIcon } from "lucide-react";
import { useState, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type {
	SearchItemResultFragment as SearchItemResultData,
	SearchRootResultFragment as SearchRootResultData,
} from "../@generated/gql/graphql";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForItem, getPathForRoot } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { Image, ImageType } from "./image";
import { LoadingText } from "./loading-text";
import { Modal, ModalBody, ModalHeader, ModalRotation } from "./modal";
import { Empty, EmptyDescription, EmptyMedia } from "./ui/empty";
import { Input } from "./ui/input";
import { Spinner } from "./ui/spinner";
import { useDebounce } from "../hooks/use-debounce";

const SearchRootResultFragment = graphql(`
	fragment SearchRootResult on RootNode {
		id
		name
		kind
		seasonCount
		episodeCount
		properties {
			posterImage {
				...ImageAsset
			}
			releasedAt
			endedAt
		}
		...GetPathForRoot
	}
`);

const SearchItemResultFragment = graphql(`
	fragment SearchItemResult on ItemNode {
		id
		name
		kind
		parent {
			name
			libraryId
		}
		properties {
			description
			thumbnailImage {
				...ImageAsset
			}
			seasonNumber
			episodeNumber
			releasedAt
			runtimeMinutes
		}
		...GetPathForItem
	}
`);

const SearchMediaQuery = graphql(`
	query SearchMedia($query: String!, $limit: Int) {
		search(query: $query, limit: $limit) {
			roots {
				...SearchRootResult
			}
			items {
				...SearchItemResult
			}
		}
	}
`);

const formatRuntime = (minutes: number | null) => {
	if (!minutes) return null;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	if (hours > 0) {
		return `${hours}h ${mins}m`;
	}
	return `${mins}m`;
};


const getRootDetail = (root: SearchRootResultData) => {
	if (root.kind === "SERIES") {
		if (root.seasonCount > 0) {
			return `${root.seasonCount} ${root.seasonCount === 1 ? "season" : "seasons"}`;
		}

		if (root.episodeCount > 0) {
			return `${root.episodeCount} ${root.episodeCount === 1 ? "episode" : "episodes"}`;
		}
	}

	return formatReleaseYear(root.properties.releasedAt, root.properties.endedAt ?? null) ?? null;
};

const SearchRootCard: FC<{
	root: FragmentType<typeof SearchRootResultFragment>;
	onSelect: () => void;
}> = ({ root: rootRef, onSelect }) => {
	const root = unmask(SearchRootResultFragment, rootRef);
	const path = getPathForRoot(root);
	const detail = getRootDetail(root);

	return (
		<Link
			to={path as never}
			onClick={onSelect}
			className="group block rounded-md p-2 transition hover:bg-zinc-900"
		>
			<div className="overflow-hidden rounded-sm">
				<Image type={ImageType.Poster} asset={root.properties.posterImage} alt={root.name} className="w-full" />
			</div>
			<div className="mt-2">
				<p className="truncate text-sm font-semibold text-zinc-100 group-hover:underline">{root.name}</p>
				{detail && <p className="text-xs text-zinc-500">{detail}</p>}
			</div>
		</Link>
	);
};

const SearchItemRow: FC<{
	item: FragmentType<typeof SearchItemResultFragment>;
	onSelect: () => void;
}> = ({ item: itemRef, onSelect }) => {
	const item = unmask(SearchItemResultFragment, itemRef);
	const path = getPathForItem(item);
	const parent = item.parent?.name ?? null;
	const runtime = formatRuntime(item.properties.runtimeMinutes);
	const index = item.properties.seasonNumber && item.properties.episodeNumber ? `S${item.properties.seasonNumber}E${item.properties.episodeNumber}` : null;

	return (
		<Link
			to={path as never}
			onClick={onSelect}
			className="group flex items-start gap-3 rounded-md p-2 transition hover:bg-zinc-900"
		>
			<div className="overflow-hidden rounded-sm">
				<Image
					type={ImageType.Thumbnail}
					asset={item.properties.thumbnailImage}
					alt={item.name}
					className="h-20"
				/>
			</div>
			<div className="min-w-0 flex-1">
				<p className="truncate text-sm font-semibold text-zinc-100 group-hover:underline">{item.name}</p>
				<p className="mt-0.5 text-xs text-zinc-500">{parent} {index} {runtime}</p>
				<p className="mt-2 line-clamp-2 text-xs text-zinc-300">
					{item.properties.description || "No description available"}
				</p>
			</div>
		</Link>
	);
};

export const SearchModal: FC<{
	open: boolean;
	onOpenChange: (open: boolean) => void;
}> = ({ open, onOpenChange }) => {
	const [query, setQuery] = useState("");
	const handleOpenChange = (nextOpen: boolean) => {
		if (!nextOpen) {
			setQuery("");
		}
		onOpenChange(nextOpen);
	};
	const [deferredQuery] = useDebounce(query.trim(), 200, 1000);
	const shouldSearch = open && deferredQuery.length > 0;
	const { data: rawData, previousData: previousRawData, loading, error } = useQuery(SearchMediaQuery, {
		skip: !shouldSearch,
		variables: {
			query: deferredQuery,
			limit: 6,
		},
	});

	const displayData = deferredQuery && !error ? (rawData || previousRawData) : rawData;
	const roots = displayData?.search.roots ?? [];
	const items = displayData?.search.items ?? [];
	const hasResults = roots.length > 0 || items.length > 0;

	return (
		<Modal
			open={open}
			onOpenChange={handleOpenChange}
			rotation={ModalRotation.Vertical}
			size="80vh"
		>
			<ModalHeader contentClassName="px-5">
				<div className="relative flex w-full items-center">
					<SearchIcon className="absolute left-0 size-4 text-zinc-500" />
					<Input
						autoFocus
						value={query}
						placeholder="Search"
						className="h-8 border-0 bg-transparent pl-8 text-sm text-zinc-100 shadow-none focus-visible:ring-0"
						onChange={(event) => setQuery(event.target.value)}
					/>
				</div>
			</ModalHeader>
			<ModalBody patterned={false} className="overflow-y-auto px-6">
				{!shouldSearch && (
					<Empty className="h-[30em] w-full">
						<EmptyMedia variant="icon">
							<SearchIcon />
						</EmptyMedia>
						<EmptyDescription>
							Whatcha' after?
						</EmptyDescription>
					</Empty>
				)}
				{shouldSearch && loading && !displayData && (
					<Empty className="h-[30em] w-full">
						<EmptyMedia variant="icon">
							<Spinner />
						</EmptyMedia>
						<EmptyDescription>
							<LoadingText />
						</EmptyDescription>
					</Empty>
				)}
				{shouldSearch && error && (
					<Empty className="h-[30em] w-full">
						<EmptyMedia variant="icon">
							<TriangleAlertIcon />
						</EmptyMedia>
						<EmptyDescription>
							{error.message || "Something went wrong"}
						</EmptyDescription>
					</Empty>
				)}
				{shouldSearch && !loading && !hasResults && (
					<Empty className="h-[30em] w-full">
						<EmptyMedia variant="icon">
							<SearchAlertIcon />
						</EmptyMedia>
						<EmptyDescription>
							I got nothin'.
						</EmptyDescription>
					</Empty>
				)}
				{hasResults && (
					<div className="space-y-6">
						{roots.length > 0 && (
							<section>
								<div className="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-3">
									{roots.map((root, index) => (
										<SearchRootCard
											key={index}
											root={root}
											onSelect={() => handleOpenChange(false)}
										/>
									))}
								</div>
							</section>
						)}
						{items.length > 0 && (
							<section>
								<div className={cn("mt-2", items.length > 0 && "space-y-1")}>
									{items.map((item, index) => (
										<SearchItemRow
											key={index}
											item={item}
											onSelect={() => handleOpenChange(false)}
										/>
									))}
								</div>
							</section>
						)}
					</div>
				)}
			</ModalBody>
		</Modal>
	);
};
