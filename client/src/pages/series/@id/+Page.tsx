import { ArrowDownNarrowWide, ChevronDown } from "lucide-react";
import { useState } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { EpisodeCard } from "../../../components/episode-card";
import { FilterButton } from "../../../components/filter-button";
import { MediaHeader } from "../../../components/media-header";
import { trpc } from "../../trpc";

export default function Page() {
	const pageContext = usePageContext();
	const mediaId = +pageContext.routeParams.id;
	const [details] = trpc.get_media_by_id.useSuspenseQuery({
		media_id: mediaId,
	});

	// todo: use details.default_connection.season_number because then it will default
	// to the last watched season once watch states are in.
	const [selectedSeasons, setSelectedSeasons] = useState<number[]>([1]);

	const [seasons] = trpc.get_seasons.useSuspenseQuery({
		show_id: mediaId,
	});

	const isAllSeasons = selectedSeasons.length === seasons.length;
	const episodesQuery = trpc.get_season_episodes.useQuery({
		show_id: mediaId,
		season_numbers: selectedSeasons,
	});

	return (
		<>
			<MediaHeader details={details} />
			<div className="flex gap-2 container mx-auto py-4 flex-wrap">
				<FilterButton
					active={isAllSeasons}
					onClick={() => {
						if (isAllSeasons) {
							setSelectedSeasons([]);
						} else {
							setSelectedSeasons(seasons);
						}
					}}
				>
					All
				</FilterButton>
				{seasons.map((season) => (
					<FilterButton
						key={season}
						active={!isAllSeasons && selectedSeasons.includes(season)}
						onClick={(event) => {
							if (event.ctrlKey) {
								setSelectedSeasons((prev) => {
									if (prev.includes(season)) {
										return prev.filter((s) => s !== season);
									}

									return [...prev, season];
								});
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
			<div className="container mx-auto pb-8">
				{episodesQuery.data && episodesQuery.data.length > 0 ? (
					<div className="space-y-2">
						{episodesQuery.data
							.sort((a, b) => {
								const seasonA = a.media.season_number || 0;
								const seasonB = b.media.season_number || 0;
								if (seasonA !== seasonB) {
									return seasonA - seasonB;
								}
								const episodeA = a.media.episode_number || 0;
								const episodeB = b.media.episode_number || 0;
								return episodeA - episodeB;
							})
							.map((episode) => (
								<EpisodeCard
									key={episode.media.id}
									episode={episode}
									showSeasonInfo={selectedSeasons.length > 1}
								/>
							))}
					</div>
				) : (
					<div className="text-center py-12 text-zinc-400">
						{isAllSeasons
							? "No episodes found for this show"
							: "No episodes found for selected seasons"}
					</div>
				)}
			</div>
		</>
	);
}
