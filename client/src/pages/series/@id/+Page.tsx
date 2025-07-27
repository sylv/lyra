import { ArrowDownNarrowWide, ChevronDown } from "lucide-react";
import { useState } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { EpisodeCard, EpisodeCardFrag } from "../../../components/episode-card";
import { FilterButton } from "../../../components/filter-button";
import { MediaHeader, MediaHeaderFrag } from "../../../components/media-header";
import { graphql } from "gql.tada";
import { useQuery, useSuspenseQuery } from "@apollo/client";
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
			id
			seasonNumber
			episodeNumber
			...EpisodeCard
		}
	}
`,
	[EpisodeCardFrag],
);

export default function Page() {
	const pageContext = usePageContext();
	const mediaId = +pageContext.routeParams.id;
	const { data } = useSuspenseQuery(Query, {
		variables: {
			mediaId: mediaId,
		},
	});

	const [selectedSeasons, setSelectedSeasons] = useQueryState<number[]>("seasons", [1]);
	const isAllSeasons = selectedSeasons.length === data.media.seasons.length;

	const { data: episodes } = useQuery(EpisodesQuery, {
		variables: {
			showId: mediaId,
			seasonNumbers: selectedSeasons,
		},
	});

	const sortedSeasons = [...data.media.seasons].sort((a, b) => a - b);
	const sortedEpisodes = [...(episodes?.mediaList ?? [])].sort((a, b) => {
		const seasonA = a.seasonNumber || 0;
		const seasonB = b.seasonNumber || 0;
		if (seasonA !== seasonB) {
			return seasonA - seasonB;
		}

		const episodeA = a.episodeNumber || 0;
		const episodeB = b.episodeNumber || 0;
		return episodeA - episodeB;
	});

	return (
		<>
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
					{sortedEpisodes[0] ? (
						<div className="space-y-2">
							{sortedEpisodes.map((episode) => (
								<EpisodeCard key={episode.id} episode={episode} showSeasonInfo={selectedSeasons.length > 1} />
							))}
						</div>
					) : (
						<div className="text-center py-12 text-zinc-400">
							{isAllSeasons ? "No episodes found for this show" : "No episodes found for selected seasons"}
						</div>
					)}
				</div>
			</div>
		</>
	);
}
