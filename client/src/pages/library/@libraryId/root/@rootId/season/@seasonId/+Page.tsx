import { useQuery } from "@apollo/client/react";
import { graphql, type VariablesOf } from "gql.tada";
import { useState } from "react";
import { Fragment } from "react/jsx-runtime";
import { usePageContext } from "vike-react/usePageContext";
import { EpisodeCard, EpisodeCardFrag, EpisodeCardSkeleton } from "../../../../../../../components/episode-card";
import { MediaFilterList } from "../../../../../../../components/media-filter-list";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../../../../../components/media-header";
import { ViewLoader } from "../../../../../../../components/view-loader";

const RootAndSeasonQuery = graphql(
	`
	query GetRootAndSeason($rootId: String!, $seasonId: String!) {
		root(rootId: $rootId) {
			id
			libraryId
			...MediaHeader
		}
		season(seasonId: $seasonId) {
			id
			name
			seasonNumber
		}
	}
`,
	[MediaHeaderFrag],
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
	const { data, loading } = useQuery(RootAndSeasonQuery, {
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

	const {
		data: episodes,
		loading: episodesLoading,
		fetchMore,
	} = useQuery(EpisodesQuery, {
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

	if (loading || !data) {
		return (
			<Fragment>
				<MediaHeaderSkeleton />
				<div className="container mx-auto py-6">
					<div className="space-y-2">
						{Array.from({ length: 6 }).map((_, index) => (
							<EpisodeCardSkeleton key={`episode-skeleton-${index}`} />
						))}
					</div>
				</div>
			</Fragment>
		);
	}

	const { root, season } = data;
	const seasonTitle = season.name || `Season ${season.seasonNumber}`;
	const rootPath = `/library/${root.libraryId}/root/${root.id}`;

	return (
		<Fragment>
			<MediaHeader media={root} />
			<div className="container mx-auto py-6">
				<div className="mb-4 flex flex-col gap-2">
					<a href={rootPath} className="text-sm text-zinc-400 hover:text-zinc-200 hover:underline">
						All seasons
					</a>
					<h2 className="text-xl font-semibold text-zinc-200">{seasonTitle}</h2>
					<div className="flex gap-2 flex-wrap">
						<MediaFilterList value={filter} onChange={(nextFilter) => setFilter({ ...filter, ...nextFilter })} />
					</div>
				</div>
				<div className="pb-8">
					{episodesLoading ? (
						<div className="space-y-2">
							{Array.from({ length: 6 }).map((_, index) => (
								<EpisodeCardSkeleton key={`episode-loading-${index}`} />
							))}
						</div>
					) : episodes?.itemList.edges[0] ? (
						<div className="space-y-2">
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
		</Fragment>
	);
}
