import { EpisodeCard } from "@/components/episode-card";
import { Image, ImageType } from "@/components/image";
import { PlayWrapper } from "@/components/play-wrapper";
import { SeasonCard } from "@/components/season-card";
import { UnplayedItemsTab } from "@/components/unplayed-items-tab";
import { useDynamicBackground } from "@/hooks/use-background";
import { useSuspenseQuery } from "@apollo/client/react";
import { Link, createFileRoute, redirect } from "@tanstack/react-router";
import { graphql } from "../@generated/gql";
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

function NodeRoute() {
	const { nodeId } = Route.useParams();
	const { data } = useSuspenseQuery(Query, { variables: { nodeId } });
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
		const episodes = sortedChildren.filter((child) => child.kind === "EPISODE");

		return (
			<div className="pt-6">
				<div className="container mx-auto flex flex-col lg:flex-row gap-6">
					<div className="shrink-0">
						<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
							<Image type={ImageType.Poster} asset={poster} alt={node.name} className="h-96" />
							<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
						</PlayWrapper>
					</div>
					<div className="flex w-full flex-col gap-2 justify-between">
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
							{episodes.length > 0 ? (
								<div className="mt-2 space-y-6">
									{episodes.map((episode) => (
										<EpisodeCard key={episode.id} episode={episode} />
									))}
								</div>
							) : (
								<div className="py-12 text-center text-zinc-400">No episodes found for this season.</div>
							)}
						</div>
					</div>
				</div>
			</div>
		);
	}

	return (
		<div className="pt-6">
			<div className="container mx-auto flex flex-col lg:flex-row gap-6">
				<div className="shrink-0">
					<PlayWrapper itemId={playableItemId} path={nodePath} watchProgress={playableWatchProgress}>
						<Image type={ImageType.Poster} asset={poster} alt={node.name} className="h-96" />
						<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
					</PlayWrapper>
				</div>
				<div className="flex w-full flex-col gap-2 justify-between">
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
				<div className="container mx-auto py-6">
					<div className="flex flex-wrap gap-4">
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
