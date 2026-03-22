import { useQuery } from "@apollo/client/react";
import { Link } from "@tanstack/react-router";
import { SearchAlertIcon, SearchIcon, TriangleAlertIcon } from "lucide-react";
import { useEffect, useRef, useState, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { SearchNodeResultFragment as SearchNodeResultData } from "../@generated/gql/graphql";
import { useDebounce } from "../hooks/use-debounce";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForNode } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { Image, ImageType } from "./image";
import { LoadingText } from "./loading-text";
import { Modal, ModalBody, ModalHeader, ModalRotation } from "./modal";
import { Empty, EmptyDescription, EmptyMedia } from "./ui/empty";
import { Input } from "./ui/input";
import { Spinner } from "./ui/spinner";

const SearchNodeResultFragment = graphql(`
	fragment SearchNodeResult on Node {
		id
		kind
		libraryId
		root {
			properties {
				displayName
			}
		}
		seasonCount
		episodeCount
		properties {
			displayName
			posterImage {
				...ImageAsset
			}
			thumbnailImage {
				...ImageAsset
			}
			description
			seasonNumber
			episodeNumber
			releasedAt
			endedAt
			runtimeMinutes
		}
		...GetPathForNode
	}
`);

const SearchMediaQuery = graphql(`
	query SearchMedia($query: String!, $limit: Int) {
		search(query: $query, limit: $limit) {
			roots {
				...SearchNodeResult
			}
			episodes {
				...SearchNodeResult
			}
		}
	}
`);

const formatRuntime = (minutes: number | null) => {
	if (!minutes) return null;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
};

const getRootDetail = (node: SearchNodeResultData) => {
	if (node.kind === "SERIES") {
		if (node.seasonCount > 0) return `${node.seasonCount} ${node.seasonCount === 1 ? "season" : "seasons"}`;
		if (node.episodeCount > 0) return `${node.episodeCount} ${node.episodeCount === 1 ? "episode" : "episodes"}`;
	}

	return formatReleaseYear(node.properties.releasedAt, node.properties.endedAt ?? null) ?? null;
};

const SearchNodeCard: FC<{ node: FragmentType<typeof SearchNodeResultFragment>; onSelect: () => void }> = ({
	node: nodeRef,
	onSelect,
}) => {
	const node = unmask(SearchNodeResultFragment, nodeRef);
	const path = getPathForNode(node);

	if (node.kind === "EPISODE") {
		const runtime = formatRuntime(node.properties.runtimeMinutes);
		const index =
			node.properties.seasonNumber && node.properties.episodeNumber
				? `S${node.properties.seasonNumber}E${node.properties.episodeNumber}`
				: null;
		return (
			<Link
				to={path}
				onClick={onSelect}
				className="group flex items-start gap-3 rounded-md p-2 transition hover:bg-zinc-900"
			>
				<div className="overflow-hidden rounded-sm">
					<Image
						type={ImageType.Thumbnail}
						asset={node.properties.thumbnailImage ?? node.properties.posterImage}
						alt={node.properties.displayName}
						className="h-20"
					/>
				</div>
				<div className="min-w-0 flex-1">
					<h3 className="truncate text-sm font-semibold text-zinc-100 group-hover:underline">{node.properties.displayName}</h3>
					<p className="mt-0.5 text-xs text-zinc-500">
						{node.root?.properties.displayName} {index} {runtime}
					</p>
					<p className="mt-2 line-clamp-2 text-xs text-zinc-300">
						{node.properties.description || "No description available"}
					</p>
				</div>
			</Link>
		);
	}

	return (
		<Link to={path} onClick={onSelect} className="group block rounded-md p-2 transition hover:bg-zinc-900">
			<div className="overflow-hidden rounded-sm">
				<Image type={ImageType.Poster} asset={node.properties.posterImage} alt={node.properties.displayName} className="w-full" />
			</div>
			<div className="mt-2">
				<h3 className="truncate text-sm font-semibold text-zinc-100 group-hover:underline">{node.properties.displayName}</h3>
				<p className="text-xs text-zinc-500">{getRootDetail(node)}</p>
			</div>
		</Link>
	);
};

export const SearchModal: FC<{ open: boolean; onOpenChange: (open: boolean) => void }> = ({ open, onOpenChange }) => {
	const [query, setQuery] = useState("");
	const inputRef = useRef<HTMLInputElement>(null);
	const handleOpenChange = (nextOpen: boolean) => {
		if (!nextOpen) setQuery("");
		onOpenChange(nextOpen);
	};
	const [deferredQuery] = useDebounce(query.trim(), 200, 1000);
	const shouldSearch = open && deferredQuery.length > 0;
	const {
		data: rawData,
		previousData,
		loading,
		error,
	} = useQuery(SearchMediaQuery, {
		skip: !shouldSearch,
		variables: { query: deferredQuery, limit: 6 },
	});
	const displayData = deferredQuery && !error ? rawData || previousData : rawData;
	const roots = displayData?.search.roots ?? [];
	const episodes = displayData?.search.episodes ?? [];
	const hasResults = roots.length > 0 || episodes.length > 0;

	useEffect(() => {
		if (open) inputRef.current?.focus();
	}, [open]);

	return (
		<Modal open={open} onOpenChange={handleOpenChange} rotation={ModalRotation.Vertical} size="80vh">
			<ModalHeader contentClassName="px-5">
				<div className="relative flex w-full items-center">
					<SearchIcon className="absolute left-0 size-4 text-zinc-500" />
					<Input
						ref={inputRef}
						value={query}
						placeholder="Search"
						className="h-8 border-0 bg-transparent pl-8 text-sm text-zinc-100 shadow-none focus-visible:ring-0"
						onChange={(event) => setQuery(event.target.value)}
					/>
				</div>
			</ModalHeader>
			<ModalBody className="overflow-y-auto px-6 text-white">
				{!shouldSearch && (
					<Empty className="h-[30em] w-full">
						<EmptyMedia variant="icon">
							<SearchIcon />
						</EmptyMedia>
						<EmptyDescription>Whatcha' after?</EmptyDescription>
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
						<EmptyDescription>{error.message || "Something went wrong"}</EmptyDescription>
					</Empty>
				)}
				{shouldSearch && !loading && !hasResults && (
					<Empty className="h-[30em] w-full">
						<EmptyMedia variant="icon">
							<SearchAlertIcon />
						</EmptyMedia>
						<EmptyDescription>I got nothin'.</EmptyDescription>
					</Empty>
				)}
				{hasResults && (
					<div className="space-y-6">
						{roots.length > 0 && (
							<section>
								<div className="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-3">
									{roots.map((node) => {
										const result = unmask(SearchNodeResultFragment, node);
										return <SearchNodeCard key={result.id} node={node} onSelect={() => handleOpenChange(false)} />;
									})}
								</div>
							</section>
						)}
						{episodes.length > 0 && (
							<section>
								<div className={cn("mt-2", episodes.length > 0 && "space-y-1")}>
									{episodes.map((node) => {
										const result = unmask(SearchNodeResultFragment, node);
										return <SearchNodeCard key={result.id} node={node} onSelect={() => handleOpenChange(false)} />;
									})}
								</div>
							</section>
						)}
					</div>
				)}
			</ModalBody>
		</Modal>
	);
};
