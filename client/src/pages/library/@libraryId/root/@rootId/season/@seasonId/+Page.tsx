import { useSuspenseQuery } from "@apollo/client/react";
import { graphql, type VariablesOf } from "gql.tada";
import { useState } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { EpisodeCard, EpisodeCardFrag } from "../../../../../../../components/episode-card";
import { Image, ImageAssetFrag, ImageType } from "../../../../../../../components/image";
import { MediaFilterList } from "../../../../../../../components/media-filter-list";
import { PlayWrapper } from "../../../../../../../components/play-wrapper";
import { ViewLoader } from "../../../../../../../components/view-loader";
import { useDynamicBackground } from "../../../../../../../hooks/use-background";
import { formatReleaseYear } from "../../../../../../../lib/format-release-year";

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

export default function Page() {
	const pageContext = usePageContext();
	const rootId = pageContext.routeParams.rootId;
	const seasonId = pageContext.routeParams.seasonId;
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

	// todo: this should maybe not use suspense? not sure if while loading more episodes
	// it will suspend. if it does we should handle that
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
	const rootPath = `/library/${root.libraryId}/root/${root.id}`;
	const seasonPath = `${rootPath}/season/${season.id}`;
	const seasonImage = season.properties.posterImage ?? season.properties.thumbnailImage;
	const dynamicAsset = season.properties.backgroundImage ?? root.properties.backgroundImage;

	useDynamicBackground(dynamicAsset);

	return (
		<div className="pt-6">
			<div className="flex gap-6 container mx-auto">
				<div>
					<PlayWrapper itemId={season.playableItem?.id} path={seasonPath} watchProgress={season.watchProgress}>
						<Image type={ImageType.Poster} asset={seasonImage} alt={seasonTitle} className="h-96" />
					</PlayWrapper>
				</div>
				<div className="flex flex-col gap-2 justify-between w-full">
					<div className="flex flex-col gap-2 mt-3">
						<a href={rootPath} className="text-sm text-zinc-400 hover:text-zinc-200 hover:underline">
							{root.name}
						</a>
						<h1 className="text-2xl font-bold">
							{seasonTitle}
							{season.properties.releasedAt && (
								<span className="text-zinc-400 ml-2 text-lg">
									{formatReleaseYear(season.properties.releasedAt, season.properties.endedAt ?? null)}
								</span>
							)}
						</h1>
						{season.properties.runtimeMinutes && (
							<p className="text-sm text-zinc-400">{season.properties.runtimeMinutes} minutes</p>
						)}
						<p className="text-sm text-zinc-400">{season.properties.description || "No description for this"}</p>
					</div>
					<div className="py-6">
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
