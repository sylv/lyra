import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { Clock } from "lucide-react";
import type { FC } from "react";
import { PlayWrapper, PlayWrapperFrag } from "./play-wrapper";
import { Thumbnail } from "./thumbnail";
import { Skeleton } from "./skeleton";

interface EpisodeCardProps {
	episode: FragmentOf<typeof EpisodeCardFrag>;
	showSeasonInfo?: boolean;
}

const formatRuntime = (minutes: number | null) => {
	if (!minutes) return null;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	if (hours > 0) {
		return `${hours}h ${mins}m`;
	}
	return `${mins}m`;
};

export const EpisodeCardFrag = graphql(
	`
	fragment EpisodeCard on Media {
		id
		name
		description
		thumbnailUrl
		seasonNumber
		episodeNumber
		runtimeMinutes
		...PlayWrapper
	}
`,
	[PlayWrapperFrag],
);

export const EpisodeCard: FC<EpisodeCardProps> = ({ episode: episodeRef, showSeasonInfo = false }) => {
	const episode = readFragment(EpisodeCardFrag, episodeRef);

	return (
		<div className="group flex gap-4 p-4 hover:bg-zinc-800/10 rounded-lg transition-colors border border-zinc-700/40">
			<div className="relative flex-shrink-0 rounded-md overflow-hidden">
				<PlayWrapper media={episode}>
					<Thumbnail imageUrl={episode.thumbnailUrl} alt={episode.name} className="aspect-[16/9] h-36 object-cover" />
				</PlayWrapper>
			</div>
			<div className="flex-1 min-w-0">
				<h3 className="font-semibold text-white mb-1">
					<span className="text-zinc-400 text-sm font-normal mr-2">
						{showSeasonInfo ? `S${episode.seasonNumber}` : ""}E{episode.episodeNumber}
					</span>
					{episode.name}
				</h3>
				{episode.runtimeMinutes && (
					<div className="flex items-center gap-1 text-sm text-zinc-400 mb-2">
						<Clock className="w-4 h-4" />
						{formatRuntime(episode.runtimeMinutes)}
					</div>
				)}
				<p className="text-sm text-zinc-300 line-clamp-3">{episode.description || "No description available"}</p>
			</div>
		</div>
	);
};

export const EpisodeCardSkeleton: FC = () => {
	return (
		<div className="group flex gap-4 p-4 hover:bg-zinc-800/10 rounded-lg transition-colors border border-zinc-700/40">
			<div className="relative flex-shrink-0 rounded-md overflow-hidden">
				<Skeleton className="aspect-[16/9] h-36 rounded-md" />
			</div>
			<div className="flex-1 min-w-0">
				<div className="mb-1">
					<Skeleton className="h-5 w-3/4" />
				</div>
				<div className="flex items-center gap-1 mb-2">
					<Skeleton className="w-4 h-4 rounded-sm" />
					<Skeleton className="h-4 w-12" />
				</div>
				<div className="space-y-1">
					<Skeleton className="h-4 w-full" />
					<Skeleton className="h-4 w-full" />
					<Skeleton className="h-4 w-2/3" />
				</div>
			</div>
		</div>
	);
};
