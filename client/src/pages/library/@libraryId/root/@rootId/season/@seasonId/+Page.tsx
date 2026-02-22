import { useSuspenseQuery } from "@apollo/client/react";
import { graphql, type VariablesOf } from "gql.tada";
import { useState } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { EpisodeCard, EpisodeCardFrag } from "../../../../../../../components/episode-card";
import { MediaFilterList } from "../../../../../../../components/media-filter-list";
import { MediaHeader, MediaHeaderFrag } from "../../../../../../../components/media-header";
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

	return (
		<div className="pt-6">
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
					{episodes?.itemList.edges[0] ? (
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
		</div>
	);
}
