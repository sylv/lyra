import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { ArrowDownNarrowWide, ChevronDown } from "lucide-react";
import { useMemo } from "react";
import { Fragment } from "react/jsx-runtime";
import { usePageContext } from "vike-react/usePageContext";
import { navigate } from "vike/client/router";
import { EpisodeCard, EpisodeCardFrag, EpisodeCardSkeleton } from "../../../components/episode-card";
import { FilterButton, FilterButtonSkeleton } from "../../../components/filter-button";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../components/media-header";

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
	query GetEpisodes($showId: Int!, $seasonNumbers: [Int!]) {
		mediaList(filter: {
			seasonNumbers: $seasonNumbers,
			parentId: $showId,
			mediaTypes: [EPISODE]
		}) {
			edges {
				node {
					id
					seasonNumber
					episodeNumber
					...EpisodeCard
				}
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

	const seasonFilter = useMemo(() => {
		if (pageContext.urlParsed.search.seasons) {
			if (pageContext.urlParsed.search.seasons === "all") {
				return null;
			}

			return pageContext.urlParsed.search.seasons.split(",").map(Number);
		}

		return [1];
	}, [pageContext.urlParsed.search.seasons]);

	const isAllSeasons = seasonFilter === null;

	const setSelectedSeasons = (seasons: number[] | null) => {
		const url = new URL(window.location.href);
		if (!seasons || seasons.length !== 0) {
			const stringified = seasons ? seasons.sort().join(",") : "all";
			url.searchParams.set("seasons", stringified);
		} else {
			url.searchParams.delete("seasons");
		}
		navigate(url.toString());
	};

	const { data: episodes, loading: episodesLoading } = useQuery(EpisodesQuery, {
		variables: {
			showId: mediaId,
			seasonNumbers: seasonFilter,
		},
	});

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

	const sortedSeasons = [...data.media.seasons].sort((a, b) => a - b);
	const sortedEpisodes = [...(episodes?.mediaList?.edges ?? [])].sort((a, b) => {
		const seasonA = a.node.seasonNumber || 0;
		const seasonB = b.node.seasonNumber || 0;
		if (seasonA !== seasonB) {
			return seasonA - seasonB;
		}

		const episodeA = a.node.episodeNumber || 0;
		const episodeB = b.node.episodeNumber || 0;
		return episodeA - episodeB;
	});

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
							active={!isAllSeasons && seasonFilter.includes(season)}
							onClick={(event) => {
								if (event.ctrlKey && seasonFilter) {
									const newSeasons = seasonFilter.includes(season)
										? seasonFilter.filter((s) => s !== season)
										: [...seasonFilter, season];

									setSelectedSeasons(newSeasons);
								} else {
									setSelectedSeasons([season]);
								}
							}}
						>
							Season {season}
						</FilterButton>
					))}
					<FilterButton onClick={() => {}}>
						<ArrowDownNarrowWide className="h-3.5 w-3.5 text-zinc-500" />
						Sort <ChevronDown className="h-3 w-3" />
					</FilterButton>
				</div>
				<div className="pb-8">
					{episodesLoading ? (
						<div className="space-y-2">
							{Array.from({ length: 6 }).map((_, i) => (
								<EpisodeCardSkeleton key={`episode-loading-${i}`} />
							))}
						</div>
					) : sortedEpisodes[0] ? (
						<div className="space-y-2">
							{sortedEpisodes.map((episode) => (
								<EpisodeCard key={episode.node.id} episode={episode.node} />
							))}
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
