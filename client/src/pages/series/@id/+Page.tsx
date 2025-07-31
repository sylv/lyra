import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { useState } from "react";
import { Fragment } from "react/jsx-runtime";
import { usePageContext } from "vike-react/usePageContext";
import { navigate } from "vike/client/router";
import type { MediaFilter } from "../../../@generated/enums";
import { EpisodeCard, EpisodeCardFrag, EpisodeCardSkeleton } from "../../../components/episode-card";
import { FilterButton, FilterButtonSkeleton } from "../../../components/filter-button";
import { MediaFilterList } from "../../../components/media-filter-list";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../components/media-header";
import { ViewLoader } from "../../../components/view-loader";

const Query = graphql(
	`
	query GetMediaById($mediaId: Int!) {
		media(mediaId: $mediaId) {
			seasons
			...MediaHeader
		}
	}
`,
	[MediaHeaderFrag],
);

const EpisodesQuery = graphql(
	`
	query GetEpisodes($filter: MediaFilter!, $after: String) {
		mediaList(filter: $filter, after: $after) {
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

export default function Page() {
	const pageContext = usePageContext();
	const mediaId = +pageContext.routeParams.id;
	const { data, loading } = useQuery(Query, {
		variables: {
			mediaId: mediaId,
		},
	});

	const [filter, setFilter] = useState<MediaFilter>(() => {
		const base: MediaFilter = {
			orderBy: "SEASON_EPISODE",
			mediaTypes: ["EPISODE"],
			parentId: mediaId,
		};

		if (pageContext.urlParsed.search.seasons) {
			if (pageContext.urlParsed.search.seasons === "all") {
				return { ...base, seasonNumbers: null };
			}

			const numbers = pageContext.urlParsed.search.seasons.split(",").map(Number);
			return { ...base, seasonNumbers: numbers };
		}

		return { ...base, seasonNumbers: [1] };
	});

	const setSelectedSeasons = (seasons: number[] | null) => {
		const url = new URL(window.location.href);
		if (!seasons || seasons.length !== 0) {
			const stringified = seasons ? seasons.sort().join(",") : "all";
			url.searchParams.set("seasons", stringified);
			setFilter({ ...filter, seasonNumbers: seasons });
		} else {
			url.searchParams.delete("seasons");
			setFilter({ ...filter, seasonNumbers: null });
		}

		navigate(url.toString());
	};

	const {
		data: episodes,
		loading: episodesLoading,
		fetchMore,
	} = useQuery(EpisodesQuery, {
		variables: {
			filter: filter,
		},
	});

	const onLoadMore = () => {
		fetchMore({
			variables: {
				after: episodes?.mediaList?.pageInfo?.endCursor,
			},
		});
	};

	if (loading || !data) {
		return (
			<Fragment>
				<MediaHeaderSkeleton />
				<div className="container mx-auto">
					<div className="flex gap-2 py-4 flex-wrap">
						{Array.from({ length: 5 }).map((_, i) => (
							<FilterButtonSkeleton key={`filter-skeleton-${i}`} />
						))}
					</div>
					<div className="pb-8">
						<div className="space-y-2">
							{Array.from({ length: 6 }).map((_, i) => (
								<EpisodeCardSkeleton key={`episode-skeleton-${i}`} />
							))}
						</div>
					</div>
				</div>
			</Fragment>
		);
	}

	const isAllSeasons = filter.seasonNumbers === null;
	const sortedSeasons = [...data.media.seasons].sort((a, b) => a - b);

	return (
		<Fragment>
			<MediaHeader media={data.media} />
			<div className="container mx-auto">
				<div className="flex gap-2 py-4 flex-wrap">
					<FilterButton
						active={isAllSeasons}
						onClick={() => {
							setSelectedSeasons(null);
						}}
					>
						All
					</FilterButton>
					{sortedSeasons.map((season) => (
						<FilterButton
							key={season}
							active={filter.seasonNumbers?.includes(season)}
							onClick={(event) => {
								if (event.ctrlKey && filter.seasonNumbers) {
									const newSeasons = filter.seasonNumbers.includes(season)
										? filter.seasonNumbers.filter((s) => s !== season)
										: [...filter.seasonNumbers, season];

									setSelectedSeasons(newSeasons);
								} else {
									setSelectedSeasons([season]);
								}
							}}
						>
							Season {season}
						</FilterButton>
					))}
					<MediaFilterList value={filter} onChange={(filter) => setFilter(filter)} />
				</div>
				<div className="pb-8">
					{episodesLoading ? (
						<div className="space-y-2">
							{Array.from({ length: 6 }).map((_, i) => (
								<EpisodeCardSkeleton key={`episode-loading-${i}`} />
							))}
						</div>
					) : episodes?.mediaList.edges[0] ? (
						<div className="space-y-2">
							{episodes.mediaList.edges.map((episode) => (
								<EpisodeCard key={episode.node.id} episode={episode.node} />
							))}
							{episodes.mediaList.pageInfo.hasNextPage && <ViewLoader onLoadMore={onLoadMore} />}
						</div>
					) : (
						<div className="text-center py-12 text-zinc-400">
							{isAllSeasons ? "No episodes found for this show" : "No episodes found for selected seasons"}
						</div>
					)}
				</div>
			</div>
		</Fragment>
	);
}
