import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { ArrowDownNarrowWide, ChevronDown } from "lucide-react";
import { Fragment } from "react/jsx-runtime";
import { usePageContext } from "vike-react/usePageContext";
import { EpisodeCard, EpisodeCardFrag, EpisodeCardSkeleton } from "../../../components/episode-card";
import { FilterButton, FilterButtonSkeleton } from "../../../components/filter-button";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../components/media-header";
import { useQueryState } from "../../../hooks/use-query-state";

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
	query GetEpisodes($showId: Int!, $seasonNumbers: [Int!]!) {
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

	const [selectedSeasons, setSelectedSeasons] = useQueryState<number[]>("seasons", [1]);

	const { data: episodes, loading: episodesLoading } = useQuery(EpisodesQuery, {
		variables: {
			showId: mediaId,
			seasonNumbers: selectedSeasons,
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

	const isAllSeasons = selectedSeasons.length === data.media.seasons.length;

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
							if (isAllSeasons) {
								setSelectedSeasons([]);
							} else {
								setSelectedSeasons(data.media.seasons);
							}
						}}
					>
						All
					</FilterButton>
					{sortedSeasons.map((season) => (
						<FilterButton
							key={season}
							active={!isAllSeasons && selectedSeasons.includes(season)}
							onClick={(event) => {
								if (event.ctrlKey) {
									const newSeasons = selectedSeasons.includes(season)
										? selectedSeasons.filter((s) => s !== season)
										: [...selectedSeasons, season];

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
								<EpisodeCard key={episode.node.id} episode={episode.node} showSeasonInfo={selectedSeasons.length > 1} />
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
