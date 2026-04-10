import { FolderPlusIcon, PlayIcon } from "lucide-react";
import { AddToCollectionModal } from "@/components/add-to-collection-modal";
import { Button, ButtonSize, ButtonStyle } from "@/components/button";
import { Image, ImageType } from "@/components/image";
import { NodeList } from "@/components/nodes/node-list";
import { PlayWrapper } from "@/components/play-wrapper";
import { SeasonCard } from "@/components/season-card";
import { UnplayedItemsTab } from "@/components/unplayed-items-tab";
import { WatchlistButton } from "@/components/watchlist-controls";
import { useDynamicBackground } from "@/hooks/use-background";
import { useState, type JSX } from "react";
import { Link, Navigate, useParams } from "react-router";
import { useQuery } from "urql";
import { graphql } from "../@generated/gql";
import { NodeAvailability, OrderBy } from "../@generated/gql/graphql";
import { useTitle } from "../hooks/use-title";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForNode } from "../lib/getPathForMedia";
import { openPlayerMedia } from "../components/player/player-context";

const Query = graphql(`
	query GetNodeById($nodeId: String!) {
		node(nodeId: $nodeId) {
			id
			libraryId
			kind
			inWatchlist
			unavailableAt
			seasonNumber
			episodeNumber
			unplayedCount
			episodeCount
			...GetPathForNode
			parent {
				id
				libraryId
				properties {
					displayName
				}
				...GetPathForNode
			}
			root {
				id
				properties {
					displayName
				}
			}
			children {
				id
				kind
				order
				properties {
					seasonNumber
				}
				...SeasonCard
			}
			properties {
				displayName
				posterImage {
					...ImageAsset
				}
				backgroundImage {
					...ImageAsset
				}
				thumbnailImage {
					...ImageAsset
				}
				firstAired
				lastAired
				runtimeMinutes
				description
			}
			watchProgress {
				id
				progressPercent
				completed
				updatedAt
			}
			nextPlayable {
				id
				watchProgress {
					id
					progressPercent
					completed
					updatedAt
				}
			}
			previousPlayable {
				id
			}
		}
	}
`);

type SeasonEntry = { id: string; seasonNumber: number | null };

export function LibraryNodeRoute() {
	const { nodeId } = useParams<{ nodeId: string }>();
	const [{ data }] = useQuery({
		query: Query,
		variables: { nodeId: nodeId! },
		context: { suspense: true },
	});

	const [view, setView] = useState<"episodes" | undefined>();
	const [isAddToCollectionOpen, setIsAddToCollectionOpen] = useState(false);
	const node = data?.node;

	const poster = node?.properties.posterImage ?? node?.properties.thumbnailImage;
	useDynamicBackground((node?.properties.backgroundImage ?? poster) || null);
	useTitle(node?.root?.properties.displayName ?? node?.properties.displayName);

	if (!node) return null;

	const playableItemId = node.nextPlayable?.id ?? (node.kind === "MOVIE" || node.kind === "EPISODE" ? node.id : null);
	const playableWatchProgress =
		node.nextPlayable?.watchProgress ?? (playableItemId === node.id ? node.watchProgress : null);
	const nodePath = getPathForNode(node);
	const parentPath = node.parent ? getPathForNode(node.parent) : null;
	const releaseYear = formatReleaseYear(node.properties.firstAired, node.properties.lastAired ?? null);
	const sortedChildren = [...node.children].sort((a, b) => {
		if (a.kind !== b.kind) {
			return a.kind === "SEASON" ? -1 : 1;
		}

		return a.order - b.order;
	});

	if (node.kind === "EPISODE" && node.parent) {
		const path = getPathForNode(node.parent);
		return <Navigate to={path} replace={true} />;
	}

	const isSeason = node.kind === "SEASON";
	const isEpisodesView = view === "episodes";
	const hasSeasons = sortedChildren.filter((c) => c.kind === "SEASON").length > 1;
	const seasonEntries: SeasonEntry[] = sortedChildren
		.filter((c) => c.kind === "SEASON")
		.map((c) => ({ id: c.id, seasonNumber: c.properties.seasonNumber }));
	const hasEpisodeChildren = sortedChildren.some((child) => child.kind === "EPISODE");
	const directAvailabilityFilter = node.unavailableAt != null ? NodeAvailability.Both : undefined;

	// Breadcrumb above the title: back button for episodes view, parent link or release year otherwise.
	let breadcrumb: JSX.Element | null = null;
	if (isEpisodesView) {
		breadcrumb = (
			<button
				type="button"
				onClick={() => setView(undefined)}
				className="-mb-2 text-left text-sm text-zinc-400 hover:text-zinc-200 hover:underline"
			>
				{node.properties.displayName}
			</button>
		);
	} else if (parentPath) {
		breadcrumb = (
			<Link to={parentPath} className="-mb-2 text-sm text-zinc-400 hover:text-zinc-200 hover:underline">
				{node.parent?.properties.displayName}
			</Link>
		);
	} else if (releaseYear) {
		breadcrumb = <span className="-mb-2 text-sm text-zinc-400">{releaseYear}</span>;
	}

	// Episode list rendered inline in the right column: by parentId for seasons, by rootId for the episodes view.
	let inlineEpisodeList: JSX.Element | null = null;
	if (isSeason) {
		inlineEpisodeList = (
			<NodeList
				type="episodes"
				defaultOrderBy={OrderBy.Order}
				filterOverride={{ parentId: node.id, availability: directAvailabilityFilter }}
			/>
		);
	} else if (isEpisodesView) {
		inlineEpisodeList = (
			<NodeList
				type="episodes"
				defaultOrderBy={OrderBy.Order}
				filterOverride={{ rootId: node.id, availability: directAvailabilityFilter }}
			/>
		);
	}

	return (
		<>
			<div className="pt-6">
				<div className="container flex flex-col lg:flex-row lg:gap-6">
					<div className="shrink-0">
						<PlayWrapper
							itemId={playableItemId}
							path={nodePath}
							unavailable={node.unavailableAt != null}
							watchProgress={playableWatchProgress}
						>
							<Image type={ImageType.Poster} asset={poster} alt={node.properties.displayName} className="h-96" />
							<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
						</PlayWrapper>
					</div>
					<div className="flex w-full flex-col gap-2">
						<div className="mt-3 flex flex-col gap-2">
							{breadcrumb}
							<h1 className="text-2xl font-bold">{isEpisodesView ? "All Episodes" : node.properties.displayName}</h1>
							<div className="flex flex-wrap items-center gap-2">
								{node.nextPlayable && (
									<Button
										style={ButtonStyle.Primary}
										size={ButtonSize.Smol}
										className="w-fit"
										icon={["play", PlayIcon]}
										iconSide="left"
										onClick={() => openPlayerMedia(node.nextPlayable!.id, true)}
									>
										{node.nextPlayable.watchProgress ? "Resume" : "Play"}
									</Button>
								)}
								<Button
									style={ButtonStyle.Glass}
									size={ButtonSize.Smol}
									className="w-fit"
									icon={["add-to-collection", FolderPlusIcon]}
									iconSide="left"
									onClick={() => setIsAddToCollectionOpen(true)}
								>
									Add to Collection
								</Button>
								<WatchlistButton nodeId={node.id} inWatchlist={node.inWatchlist} />
							</div>
							{!isEpisodesView && node.properties.runtimeMinutes && (
								<p className="text-sm text-zinc-400">{node.properties.runtimeMinutes} minutes</p>
							)}
							{!isEpisodesView && (
								<p className="text-sm text-zinc-400">
									{node.properties.description || (!isSeason ? "No description available" : null)}
								</p>
							)}
						</div>
						{inlineEpisodeList && <div className="pb-16">{inlineEpisodeList}</div>}
					</div>
				</div>
				{!isSeason && !isEpisodesView && (
					<>
						{hasEpisodeChildren && seasonEntries.length === 0 ? (
							<div className="container py-6">
								<NodeList
									type="episodes"
									defaultOrderBy={OrderBy.Order}
									filterOverride={{ rootId: node.id, availability: directAvailabilityFilter }}
								/>
							</div>
						) : null}
						{sortedChildren.length > 0 && seasonEntries.length > 0 && (
							<div className="container py-6">
								<div className="flex flex-wrap gap-4">
									{hasSeasons && (
										<div className="flex w-38 flex-col gap-2 overflow-hidden">
											<PlayWrapper
												itemId={playableItemId}
												path={nodePath}
												unavailable={node.unavailableAt != null}
												watchProgress={playableWatchProgress}
											>
												<Image type={ImageType.Poster} asset={poster} alt="All Episodes" className="w-full" />
												<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
											</PlayWrapper>
											<button
												type="button"
												onClick={() => setView("episodes")}
												className="block w-full truncate text-left text-sm group"
											>
												<span className="group-hover:underline">All Episodes</span>
												{node.episodeCount > 0 && (
													<p className="-mt-0.5 text-xs text-zinc-500">
														{node.episodeCount} {node.episodeCount === 1 ? "episode" : "episodes"}
													</p>
												)}
											</button>
										</div>
									)}
									{sortedChildren.map((child) =>
										child.kind === "SEASON" ? <SeasonCard key={child.id} season={child} /> : null,
									)}
								</div>
							</div>
						)}
					</>
				)}
			</div>
			<AddToCollectionModal nodeId={node.id} open={isAddToCollectionOpen} onOpenChange={setIsAddToCollectionOpen} />
		</>
	);
}
