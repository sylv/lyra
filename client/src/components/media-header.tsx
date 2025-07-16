import type { FC } from "react";
import type { MediaDetails } from "../@generated/server";
import { PlayWrapper } from "./play-wrapper";
import { Poster } from "./poster";
import TMDBLogo from "../../assets/tmdb-primary-short-blue.svg";

interface MediaHeaderProps {
	details: MediaDetails;
}

const formatYear = (input: number) => {
	const date = new Date(input * 1000);
	return date.getFullYear();
};

const getTMDBUrl = (mediaType: string, tmdbParentId: number) => {
	const baseUrl = "https://www.themoviedb.org";
	switch (mediaType) {
		case "Movie":
			return `${baseUrl}/movie/${tmdbParentId}`;
		case "Show":
			return `${baseUrl}/tv/${tmdbParentId}`;
		default:
			throw new Error(`Unknown media type: ${mediaType}`);
	}
};

export const MediaHeader: FC<MediaHeaderProps> = ({ details }) => {
	return (
		<div className="bg-zinc-900 py-6">
			<div className="flex gap-6 container mx-auto">
				<PlayWrapper media={details}>
					<Poster
						imageUrl={details.media.poster_url}
						alt={details.media.name}
					/>
				</PlayWrapper>
				<div className="flex flex-col gap-2 justify-between">
					<div className="flex flex-col gap-2">
						<h1 className="text-2xl font-bold">
							{details.media.name}
							{details.media.release_date && (
								<span className="text-zinc-400 ml-2 text-lg">
									{formatYear(details.media.release_date)}
								</span>
							)}
						</h1>
						{details.media.runtime_minutes && (
							<p className="text-sm text-zinc-400">
								{details.media.runtime_minutes} minutes
							</p>
						)}
						<p className="text-sm text-zinc-400">
							{details.media.description || "No description for this"}
						</p>
					</div>
					<div className="flex gap-2">
						<a
							className="bg-zinc-950 px-4 py-1 rounded-lg flex items-center gap-2 text-sm text-zinc-400 hover:bg-zinc-950/60 hover:text-zinc-300 transition-colors"
							target="_blank"
							rel="noreferrer"
							href={getTMDBUrl(
								details.media.media_type,
								details.media.tmdb_parent_id,
							)}
						>
							<img src={TMDBLogo} alt="TMDB Logo" className="h-8 w-8" />
							{details.media.rating && (
								<span className="ml-2">
									{(details.media.rating * 10).toFixed(1)}%
								</span>
							)}
						</a>
					</div>
				</div>
			</div>
		</div>
	);
};
