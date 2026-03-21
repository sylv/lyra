import { EpisodeCard } from "@/components/episode-card";
import { FilterButton } from "@/components/filter-button";
import { Image, ImageType } from "@/components/image";
import { NodeFilterList } from "@/components/nodes/node-filter-list";
import { PlayWrapper } from "@/components/play-wrapper";
import { SeasonCard } from "@/components/season-card";
import { UnplayedItemsTab } from "@/components/unplayed-items-tab";
import { useDynamicBackground } from "@/hooks/use-background";
import { useSuspenseQuery } from "@apollo/client/react";
import { Link, createFileRoute, redirect } from "@tanstack/react-router";
import { Suspense, useState } from "react";
import { graphql } from "../@generated/gql";
import { NodeKind, OrderBy, type NodeFilter } from "../@generated/gql/graphql";
import { getApolloClient } from "../client";
import { getPathForNodeData } from "../lib/getPathForMedia";
import { useTitle } from "../hooks/use-title";
import { formatReleaseYear } from "../lib/format-release-year";

const Query = graphql(`
	query GetNodeById($nodeId: String!) {
		node(nodeId: $nodeId) {
			id
			libraryId
			kind
			name
			seasonNumber
			episodeNumber
			parent {
				id
				name
				libraryId
			}
			root {
				id
				name
			}
			children {
				id
				kind
				order
				properties {
					seasonNumber
				}
				...SeasonCard
				...EpisodeCard
			}
			properties {
				posterImage {
					...ImageAsset
				}
				backgroundImage {
					...ImageAsset
				}
				thumbnailImage {
					...ImageAsset
				}
				releasedAt
				endedAt
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

const EpisodesQuery = graphql(`
	query GetEpisodes($filter: NodeFilter!, $first: Int) {
		nodeList(filter: $filter, first: $first) {
			edges {
				node {
					id
					...EpisodeCard
				}
			}
		}
	}
`);

export const Route = createFileRoute("/library_/$libraryId/node_/$nodeId")({
	component: NodeRoute,
	loader: async ({ params }) => {
		const { data } = await getApolloClient().query({
			query: Query,
			variables: { nodeId: params.nodeId },
		});

		const node = data?.node;
		if (node?.kind === "EPISODE" && node.parent) {
			const path = getPathForNodeData({
				id: node.parent.id,
				libraryId: node.parent.libraryId,
				__typename: "Node",
			});

			throw redirect({
				to: path,
				replace: true,
			});
		}
	},
});

type SeasonEntry = { id: string; seasonNumber: number | null };

// episode list with NodeFilterList controls and an optional season filter.
// for season view: pass parentId only.
// for all-episodes view: pass rootId + seasons; season filter overrides to parentId when active.
function EpisodeListView({
	rootId,
	parentId,
	seasons,
}: {
	rootId?: string;
	parentId?: string;
	seasons?: SeasonEntry[];
}) {
	const [orderBy, setOrderBy] = useState<OrderBy | undefined>();
	const [watched, setWatched] = useState<boolean | null>(null);
	const [season, setSeason] = useState<string | undefined>();

	const filter: NodeFilter =
		parentId != null
			? { parentId, kinds: [NodeKind.Episode], orderBy, watched }
			: season != null
				? { parentId: season, kinds: [NodeKind.Episode], orderBy, watched }
				: { rootId, kinds: [NodeKind.Episode], orderBy, watched };

	const { data } = useSuspenseQuery(EpisodesQuery, { variables: { filter, first: 500 } });
	const episodes = data.nodeList.edges.map((e) => e.node);

	const handleFilterChange = (newFilter: NodeFilter) => {
		setOrderBy(newFilter.orderBy ?? undefined);
		setWatched(newFilter.watched ?? null);
	};

	return (
		<div className="space-y-4">
			<div className="flex flex-wrap gap-2">
				<NodeFilterList value={{ orderBy: orderBy ?? OrderBy.Order, watched }} onChange={handleFilterChange} />
				{seasons && seasons.length > 1 && (
					<>
						<FilterButton active={season == null} onClick={() => setSeason(undefined)}>
							All seasons
						</FilterButton>
						{seasons.map((entry) => (
							<FilterButton key={entry.id} active={season === entry.id} onClick={() => setSeason(entry.id)}>
								Season {entry.seasonNumber}
							</FilterButton>
						))}
					</>
				)}
			</div>
			{episodes.length > 0 ? (
				<div className="space-y-6">
					{episodes.map((episode) => (
						<EpisodeCard key={episode.id} episode={episode} />
					))}
				</div>
			) : (
				<div className="py-12 text-center text-zinc-400">No episodes found.</div>
			)}
		</div>
	);
}

function NodeRoute() {
	const { nodeId } = Route.useParams();
	const { data } = useSuspenseQuery(Query, { variables: { nodeId } });
	const [view, setView] = useState<"episodes" | undefined>();
	const node = data.node;
	if (node == null) {
		return null;
	}
	const playableItemId = node.nextPlayable?.id ?? (node.kind === "MOVIE" || node.kind === "EPISODE" ? node.id : null);
	const playableWatchProgress =
		node.nextPlayable?.watchProgress ?? (playableItemId === node.id ? node.watchProgress : null);
	const poster = node.properties.posterImage ?? node.properties.thumbnailImage;
	const nodePath = getPathForNodeData({ id: node.id, libraryId: node.libraryId, __typename: "Node" });
	const parentPath = node.parent
		? getPathForNodeData({
				id: node.parent.id,
				libraryId: node.parent.libraryId,
				__typename: "Node",
			})
		: null;
	const releaseYear = formatReleaseYear(node.properties.releasedAt, node.properties.endedAt ?? null);
	const sortedChildren = [...node.children].sort((a, b) => {
		if (a.kind !== b.kind) {
			return a.kind === "SEASON" ? -1 : 1;
		}

		return a.order - b.order;
	});

	useDynamicBackground(node.properties.backgroundImage ?? poster);
	useTitle(node.root?.name ?? node.name);

	if (node.kind === "SEASON") {
		return (
			<div className="pt-6">
				<div className="container flex flex-col lg:flex-row lg:gap-6">
					<div className="shrink-0">
						<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
							<Image type={ImageType.Poster} asset={poster} alt={node.name} className="h-96" />
							<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
						</PlayWrapper>
					</div>
					<div className="flex w-full flex-col gap-2">
						<div className="mt-3 flex flex-col gap-2">
							{parentPath && (
								<Link to={parentPath} className="-mb-2 text-sm text-zinc-400 hover:text-zinc-200 hover:underline">
									{node.parent?.name}
								</Link>
							)}
							<h1 className="text-2xl font-bold">{node.name}</h1>
							{node.properties.runtimeMinutes && (
								<p className="text-sm text-zinc-400">{node.properties.runtimeMinutes} minutes</p>
							)}
							<p className="text-sm text-zinc-400">{node.properties.description}</p>
						</div>
						<div className="pb-16">
							<Suspense>
								<EpisodeListView parentId={nodeId} />
							</Suspense>
						</div>
					</div>
				</div>
			</div>
		);
	}

	const hasSeasons = sortedChildren.some((c) => c.kind === "SEASON");
	const seasonEntries: SeasonEntry[] = sortedChildren
		.filter((c) => c.kind === "SEASON")
		.map((c) => ({ id: c.id, seasonNumber: c.properties.seasonNumber }));

	if (view === "episodes") {
		return (
			<div className="pt-6">
				<div className="container flex flex-col lg:flex-row lg:gap-6">
					<div className="shrink-0">
						<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
							<Image type={ImageType.Poster} asset={poster} alt={node.name} className="h-96" />
							<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
						</PlayWrapper>
					</div>
					<div className="flex w-full flex-col gap-2">
						<div className="mt-3 flex flex-col gap-2">
							<button
								type="button"
								onClick={() => setView(undefined)}
								className="-mb-2 text-sm text-zinc-400 hover:text-zinc-200 hover:underline text-left"
							>
								{node.name}
							</button>
							<h1 className="text-2xl font-bold">All Episodes</h1>
						</div>
						<div className="pb-16">
							<Suspense>
								<EpisodeListView rootId={node.id} seasons={seasonEntries} />
							</Suspense>
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
						<Image type={ImageType.Poster} asset={poster} alt={node.name} className="h-96" />
						<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
					</PlayWrapper>
				</div>
				<div className="flex w-full flex-col gap-2">
					<div className="mt-3 flex flex-col gap-2">
						{parentPath ? (
							<Link to={parentPath} className="-mb-2 text-sm text-zinc-400 hover:text-zinc-200 hover:underline">
								{node.parent?.name}
							</Link>
						) : releaseYear ? (
							<span className="-mb-2 text-sm text-zinc-400">{releaseYear}</span>
						) : null}
						<h1 className="text-2xl font-bold">{node.name}</h1>
						{node.properties.runtimeMinutes && (
							<p className="text-sm text-zinc-400">{node.properties.runtimeMinutes} minutes</p>
						)}
						<p className="text-sm text-zinc-400">{node.properties.description || "No description available"}</p>
					</div>
				</div>
			</div>
			{sortedChildren.length > 0 && (
				<div className="container py-6">
					<div className="flex flex-wrap gap-4">
						{hasSeasons && (
							<div className="flex flex-col gap-2 overflow-hidden w-38">
								<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
									<Image type={ImageType.Poster} asset={poster} alt="All Episodes" className="w-full" />
									<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
								</PlayWrapper>
								<button
									type="button"
									onClick={() => setView("episodes")}
									className="block w-full truncate text-sm group text-left"
								>
									<span className="group-hover:underline">All Episodes</span>
									{node.episodeCount > 0 && (
										<p className="text-xs text-zinc-500 -mt-0.5">
											{node.episodeCount} {node.episodeCount === 1 ? "episode" : "episodes"}
										</p>
									)}
								</button>
							</div>
						)}
						{sortedChildren.map((child) =>
							child.kind === "SEASON" ? (
								<SeasonCard key={child.id} season={child} />
							) : (
								<div key={child.id} className="w-full">
									<EpisodeCard episode={child} />
								</div>
							),
						)}
					</div>
				</div>
			)}
		</div>
	);
}
