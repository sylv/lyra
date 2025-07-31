import type { FC } from "react";
import { PlayWrapper, PlayWrapperFrag } from "./play-wrapper";
import { Poster } from "./poster";
import TMDBLogo from "../../assets/tmdb-primary-short-blue.svg";
import { useDynamicBackground } from "../hooks/use-background";
import { getImageProxyUrl } from "../lib/getImageProxyUrl";
import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type { MediaType } from "../@generated/enums";
import { Skeleton } from "./skeleton";
import { formatReleaseYear } from "../lib/format-release-year";

interface MediaHeaderProps {
	media: FragmentOf<typeof MediaHeaderFrag>;
}

const getTMDBUrl = (mediaType: MediaType, tmdbParentId: number) => {
	const baseUrl = "https://www.themoviedb.org";
	switch (mediaType) {
		case "MOVIE":
			return `${baseUrl}/movie/${tmdbParentId}`;
		case "SHOW":
			return `${baseUrl}/tv/${tmdbParentId}`;
		default:
			throw new Error(`Unknown media type: ${mediaType}`);
	}
};

export const MediaHeaderFrag = graphql(
	`
	fragment MediaHeader on Media {
		id
		name
		mediaType
		posterUrl
		backgroundUrl
		startDate
		endDate
		runtimeMinutes
		description
		tmdbParentId
		rating
		...PlayWrapper
	}
`,
	[PlayWrapperFrag],
);

export const MediaHeader: FC<MediaHeaderProps> = ({ media: mediaRaw }) => {
	const media = readFragment(MediaHeaderFrag, mediaRaw);
	const dynamicUrl = media.backgroundUrl && getImageProxyUrl(media.backgroundUrl, 200);

	useDynamicBackground(dynamicUrl);

	return (
		<div className="bg-zinc-800/30 border-700/30 p-6 border-b">
			<div className="flex gap-6 container mx-auto">
				<PlayWrapper media={media}>
					<Poster imageUrl={media.posterUrl} alt={media.name} className="h-72" />
				</PlayWrapper>
				<div className="flex flex-col gap-2 justify-between">
					<div className="flex flex-col gap-2">
						<h1 className="text-2xl font-bold">
							{media.name}
							{media.startDate && (
								<span className="text-zinc-400 ml-2 text-lg">{formatReleaseYear(media.startDate, media.endDate)}</span>
							)}
						</h1>
						{media.runtimeMinutes && <p className="text-sm text-zinc-400">{media.runtimeMinutes} minutes</p>}
						<p className="text-sm text-zinc-400">{media.description || "No description for this"}</p>
					</div>
					<div className="flex gap-2">
						<a
							className="bg-zinc-700/30 px-4 py-1 rounded-lg flex items-center gap-2 text-sm text-zinc-400 hover:bg-zinc-700/50 hover:text-zinc-300 transition-colors"
							target="_blank"
							rel="noreferrer"
							href={getTMDBUrl(media.mediaType, media.tmdbParentId)}
						>
							<img src={TMDBLogo} alt="TMDB Logo" className="h-8 w-8" />
							{media.rating && <span className="ml-2">{(media.rating * 10).toFixed(1)}%</span>}
						</a>
					</div>
				</div>
			</div>
		</div>
	);
};

export const MediaHeaderSkeleton: FC = () => {
	return (
		<div className="bg-zinc-800/30 border-700/30 p-6 border-b">
			<div className="flex gap-6 container mx-auto">
				<Skeleton className="h-72 aspect-[2/3] rounded-md" />
				<div className="flex flex-col gap-2 justify-between">
					<div className="flex flex-col gap-2">
						<div className="flex items-center gap-2">
							<Skeleton className="h-8 w-64" />
							<Skeleton className="h-6 w-20" />
						</div>
						<Skeleton className="h-4 w-32" />
						<div className="space-y-1">
							<Skeleton className="h-4 w-full" />
							<Skeleton className="h-4 w-full" />
							<Skeleton className="h-4 w-3/4" />
						</div>
					</div>
					<div className="flex gap-2">
						<Skeleton className="h-10 w-32 rounded-lg" />
					</div>
				</div>
			</div>
		</div>
	);
};
