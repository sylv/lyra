import { Image, ImageType } from "@/components/image";
import { NodeList } from "@/components/nodes/node-list";
import { PlayWrapper } from "@/components/play-wrapper";
import { SeasonCard } from "@/components/season-card";
import { UnplayedItemsTab } from "@/components/unplayed-items-tab";
import { useDynamicBackground } from "@/hooks/use-background";
import { useState } from "react";
import { Link, Navigate, useParams } from "react-router";
import { useQuery } from "urql";
import { graphql } from "../@generated/gql";
import { useTitle } from "../hooks/use-title";
import { formatReleaseYear } from "../lib/format-release-year";
import { getPathForNodeData } from "../lib/getPathForMedia";

const Query = graphql(`
	query GetNodeById($nodeId: String!) {
		node(nodeId: $nodeId) {
			id
			libraryId
			kind
			seasonNumber
			episodeNumber
			parent {
				id
				libraryId
				properties {
					displayName
				}
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
				progressPercent
				completed
				updatedAt
			}
			nextPlayable {
				id
				watchProgress {
					progressPercent
					completed
					updatedAt
				}
			}
			previousPlayable {
				id
			}
			unplayedCount
			episodeCount
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
	const node = data?.node;

	const poster = node?.properties.posterImage ?? node?.properties.thumbnailImage;
	useDynamicBackground((node?.properties.backgroundImage ?? poster) || null);
	useTitle(node?.root?.properties.displayName ?? node?.properties.displayName);

	if (!node) return null;

	const playableItemId = node.nextPlayable?.id ?? (node.kind === "MOVIE" || node.kind === "EPISODE" ? node.id : null);
	const playableWatchProgress =
		node.nextPlayable?.watchProgress ?? (playableItemId === node.id ? node.watchProgress : null);
	const nodePath = getPathForNodeData({ id: node.id, libraryId: node.libraryId, __typename: "Node" });
	const parentPath = node.parent
		? getPathForNodeData({
				id: node.parent.id,
				libraryId: node.parent.libraryId,
				__typename: "Node",
			})
		: null;
	const releaseYear = formatReleaseYear(node.properties.firstAired, node.properties.lastAired ?? null);
	const sortedChildren = [...node.children].sort((a, b) => {
		if (a.kind !== b.kind) {
			return a.kind === "SEASON" ? -1 : 1;
		}

		return a.order - b.order;
	});

	if (node.kind === "EPISODE" && node.parent) {
		const path = getPathForNodeData({ id: node.parent.id, libraryId: node.parent.libraryId, __typename: "Node" });
		return <Navigate to={path} replace={true} />;
	}

	if (node.kind === "SEASON") {
		return (
			<div className="pt-6">
				<div className="container flex flex-col lg:flex-row lg:gap-6">
					<div className="shrink-0">
						<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
							<Image type={ImageType.Poster} asset={poster} alt={node.properties.displayName} className="h-96" />
							<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
						</PlayWrapper>
					</div>
					<div className="flex w-full flex-col gap-2">
						<div className="mt-3 flex flex-col gap-2">
							{parentPath && (
								<Link to={parentPath} className="-mb-2 text-sm text-zinc-400 hover:text-zinc-200 hover:underline">
									{node.parent?.properties.displayName}
								</Link>
							)}
							<h1 className="text-2xl font-bold">{node.properties.displayName}</h1>
							{node.properties.runtimeMinutes && (
								<p className="text-sm text-zinc-400">{node.properties.runtimeMinutes} minutes</p>
							)}
							<p className="text-sm text-zinc-400">{node.properties.description}</p>
						</div>
						<div className="pb-16">
							<NodeList type="episodes" filterOverride={{ parentId: node.id }} />
						</div>
					</div>
				</div>
			</div>
		);
	}

	const hasSeasons = sortedChildren.filter((c) => c.kind === "SEASON").length > 1;
	const seasonEntries: SeasonEntry[] = sortedChildren
		.filter((c) => c.kind === "SEASON")
		.map((c) => ({ id: c.id, seasonNumber: c.properties.seasonNumber }));
	const hasEpisodeChildren = sortedChildren.some((child) => child.kind === "EPISODE");

	if (view === "episodes") {
		return (
			<div className="pt-6">
				<div className="container flex flex-col lg:flex-row lg:gap-6">
					<div className="shrink-0">
						<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
							<Image type={ImageType.Poster} asset={poster} alt={node.properties.displayName} className="h-96" />
							<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
						</PlayWrapper>
					</div>
					<div className="flex w-full flex-col gap-2">
						<div className="mt-3 flex flex-col gap-2">
							<button
								type="button"
								onClick={() => setView(undefined)}
								className="-mb-2 text-left text-sm text-zinc-400 hover:text-zinc-200 hover:underline"
							>
								{node.properties.displayName}
							</button>
							<h1 className="text-2xl font-bold">All Episodes</h1>
						</div>
						<div className="pb-16">
							<NodeList type="episodes" filterOverride={{ rootId: node.id }} />
						</div>
					</div>
				</div>
			</div>
		);
	}

	return (
		<div className="pt-6">
			<div className="container flex flex-col lg:flex-row lg:gap-6">
				<div className="shrink-0">
					<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
						<Image type={ImageType.Poster} asset={poster} alt={node.properties.displayName} className="h-96" />
						<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
					</PlayWrapper>
				</div>
				<div className="flex w-full flex-col gap-2">
					<div className="mt-3 flex flex-col gap-2">
						{parentPath ? (
							<Link to={parentPath} className="-mb-2 text-sm text-zinc-400 hover:text-zinc-200 hover:underline">
								{node.parent?.properties.displayName}
							</Link>
						) : releaseYear ? (
							<span className="-mb-2 text-sm text-zinc-400">{releaseYear}</span>
						) : null}
						<h1 className="text-2xl font-bold">{node.properties.displayName}</h1>
						{node.properties.runtimeMinutes && (
							<p className="text-sm text-zinc-400">{node.properties.runtimeMinutes} minutes</p>
						)}
						<p className="text-sm text-zinc-400">{node.properties.description || "No description available"}</p>
					</div>
				</div>
			</div>
			{hasEpisodeChildren && seasonEntries.length === 0 ? (
				<div className="container py-6">
					<NodeList type="episodes" filterOverride={{ rootId: node.id }} />
				</div>
			) : null}
			{sortedChildren.length > 0 && seasonEntries.length > 0 && (
				<div className="container py-6">
					<div className="flex flex-wrap gap-4">
						{hasSeasons && (
							<div className="flex w-38 flex-col gap-2 overflow-hidden">
								<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
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
		</div>
	);
}
