import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { useMemo, type FC } from "react";
import TMDBLogo from "../../assets/tmdb-primary-short-blue.svg";
import { cn } from "../lib/utils";

interface TMDBRatingPillProps {
	media: FragmentOf<typeof TMDBRatingPillFrag>;
	mini?: boolean;
}

export const TMDBRatingPillFrag = graphql(`
	fragment TMDBRatingPill on Media {
		kind
		tmdbId
		rating
        seasonNumber
        episodeNumber
		parent {
			tmdbId
		}
	}
`);

export const TMDBRatingPill: FC<TMDBRatingPillProps> = ({ media: mediaRaw, mini }) => {
	const media = readFragment(TMDBRatingPillFrag, mediaRaw);
	const url = useMemo(() => {
		const tmdbId = media.parent ? media.parent.tmdbId : media.tmdbId;
		if (media.kind === "MOVIE") {
			return `https://www.themoviedb.org/movie/${tmdbId}`;
		}

		if (media.kind === "SHOW") {
			return `https://www.themoviedb.org/tv/${tmdbId}`;
		}

		if (media.kind === "EPISODE") {
			return `https://www.themoviedb.org/tv/${tmdbId}/season/${media.seasonNumber}/episode/${media.episodeNumber}`;
		}

		throw new Error(`Do not know how to get TMDb url for ${media.kind}`);
	}, [media]);

	return (
		<a
			target="_blank"
			rel="noreferrer"
			href={url}
			className={cn(
				"bg-zinc-700/30 px-4 py-1 rounded-lg flex items-center gap-2 text-sm text-zinc-400 hover:bg-zinc-700/50 hover:text-zinc-300 transition-colors",
				mini && "px-2 py-0.5 text-xs",
			)}
		>
			<img src={TMDBLogo} alt="TMDB Logo" className={cn("h-8 w-8", mini && "h-5 w-5")} />
			{media.rating && <span className={cn(mini ? "ml-1" : "ml-2")}>{(media.rating * 10).toFixed(0)}%</span>}
		</a>
	);
};
