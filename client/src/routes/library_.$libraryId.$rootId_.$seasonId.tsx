import { useSuspenseQuery } from "@apollo/client/react";
import { Link, createFileRoute } from "@tanstack/react-router";
import { graphql, type VariablesOf } from "gql.tada";
import { useState } from "react";
import { EpisodeCard, EpisodeCardFrag } from "@/components/episode-card";
import { Image, ImageAssetFrag, ImageType } from "@/components/image";
import { MediaFilterList } from "@/components/media-filter-list";
import { PlayWrapper } from "@/components/play-wrapper";
import { ViewLoader } from "@/components/view-loader";
import { UnplayedItemsTab } from "@/components/unplayed-items-tab";
import { useDynamicBackground } from "@/hooks/use-background";

const RootAndSeasonQuery = graphql(
	`
	query GetRootAndSeason($rootId: String!, $seasonId: String!) {
		root(rootId: $rootId) {
			id
			libraryId
			name
			properties {
				backgroundImage {
					...ImageAsset
				}
				posterImage {
					...ImageAsset
				}
			}
		}
		season(seasonId: $seasonId) {
			id
			name
			seasonNumber
			properties {
				posterImage {
					...ImageAsset
				}
				thumbnailImage {
					...ImageAsset
				}
				backgroundImage {
					...ImageAsset
				}
				releasedAt
				endedAt
				runtimeMinutes
				description
			}
			playableItem {
				id
			}
			watchProgress {
				progressPercent
				updatedAt
			}
			unplayedItems
		}
	}
`,
	[ImageAssetFrag],
);

const EpisodesQuery = graphql(
	`
	query GetSeasonEpisodes($filter: ItemNodeFilter!, $after: String) {
		itemList(filter: $filter, after: $after) {
			edges {
				node {
					id
					...EpisodeCard
				}
			}
			pageInfo {
				endCursor
				hasNextPage
			}
		}
	}
`,
	[EpisodeCardFrag],
);

type ItemNodeFilter = VariablesOf<typeof EpisodesQuery>["filter"];
type EpisodeListFilter = Pick<ItemNodeFilter, "orderBy" | "orderDirection" | "watched">;

export const Route = createFileRoute("/library_/$libraryId/$rootId_/$seasonId")({
	component: SeasonRoute,
});

function SeasonRoute() {
	const { rootId, seasonId } = Route.useParams();
	const [filter, setFilter] = useState<EpisodeListFilter>({
		orderBy: "SEASON_EPISODE",
	});
	const { data } = useSuspenseQuery(RootAndSeasonQuery, {
		variables: {
			rootId,
			seasonId,
		},
	});

	const episodeFilter: ItemNodeFilter = {
		rootId,
		seasonNumbers: [data?.season.seasonNumber ?? -1],
		...filter,
	};

	const { data: episodes, fetchMore } = useSuspenseQuery(EpisodesQuery, {
		variables: {
			filter: episodeFilter,
		},
		skip: !data,
	});

	const onLoadMore = () => {
		fetchMore({
			variables: {
				after: episodes?.itemList?.pageInfo?.endCursor,
			},
		});
	};

	const { root, season } = data;
	const seasonTitle = season.name || `Season ${season.seasonNumber}`;
	const rootPath = `/library/${root.libraryId}/${root.id}`;
	const seasonPath = `${rootPath}/${season.id}`;
	const seasonImage = season.properties.posterImage ?? season.properties.thumbnailImage;

	const dynamicAsset = root.properties.backgroundImage || root.properties.posterImage;
	useDynamicBackground(dynamicAsset);

	return (
		<div className="pt-6">
			<div className="flex gap-6 container mx-auto">
				<div className="shrink-0">
					<PlayWrapper itemId={season.playableItem?.id} path={seasonPath} watchProgress={season.watchProgress}>
						<Image type={ImageType.Poster} asset={seasonImage} alt={seasonTitle} className="h-96" />
						<UnplayedItemsTab count={season.unplayedItems} />
					</PlayWrapper>
				</div>
				<div className="flex flex-col gap-2 justify-between w-full">
					<div className="flex flex-col gap-2 mt-3">
						<Link to={rootPath as never} className="text-sm text-zinc-400 hover:text-zinc-200 hover:underline -mb-2">
							{root.name}
						</Link>
						<h1 className="text-2xl font-bold">{seasonTitle}</h1>
						{season.properties.runtimeMinutes && (
							<p className="text-sm text-zinc-400">{season.properties.runtimeMinutes} minutes</p>
						)}
					</div>
					<div className="pb-6">
						<div className="mb-4 flex gap-2 flex-wrap">
							<MediaFilterList value={filter} onChange={(nextFilter) => setFilter({ ...filter, ...nextFilter })} />
						</div>
						<div className="pb-16">
							{episodes?.itemList.edges[0] ? (
								<div className="space-y-4">
									{episodes.itemList.edges.map((episode) => (
										<EpisodeCard key={episode.node.id} episode={episode.node} />
									))}
									{episodes.itemList.pageInfo.hasNextPage && <ViewLoader onLoadMore={onLoadMore} />}
								</div>
							) : (
								<div className="text-center py-12 text-zinc-400">No episodes found for this season.</div>
							)}
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
