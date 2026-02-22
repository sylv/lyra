import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { Clock } from "lucide-react";
import type { FC } from "react";
import { getPathForItem, GetPathForItemFrag } from "../lib/getPathForMedia";
import { PlayWrapper } from "./play-wrapper";
import { Thumbnail } from "./thumbnail";

interface EpisodeCardProps {
	episode: FragmentOf<typeof EpisodeCardFrag>;
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
	fragment EpisodeCard on ItemNode {
		id
		name
		properties {
			description
			thumbnailUrl
			seasonNumber
			episodeNumber
			runtimeMinutes
		}
		watchProgress {
			progressPercent
			updatedAt
		}
		...GetPathForItem
	}
`,
	[GetPathForItemFrag],
);

export const EpisodeCard: FC<EpisodeCardProps> = ({ episode: episodeRef }) => {
	const episode = readFragment(EpisodeCardFrag, episodeRef);
	const path = getPathForItem(episode);

	return (
		<div className="group flex gap-4 p-4 hover:bg-zinc-800/10 rounded-lg transition-colors border border-zinc-700/40">
			<div className="relative shrink-0 overflow-hidden rounded-md">
				<PlayWrapper itemId={episode.id} path={path} watchProgress={episode.watchProgress}>
					<Thumbnail imageUrl={episode.properties.thumbnailUrl} alt={episode.name} className="h-36 " />
				</PlayWrapper>
			</div>
			<div className="flex flex-col justify-between gap-2">
				<div className="flex-1 min-w-0">
					<h3 className="font-semibold text-white mb-1">
						<span className="text-zinc-400 text-sm font-normal mr-2">
							S{episode.properties.seasonNumber}E{episode.properties.episodeNumber}
						</span>
						{episode.name}
					</h3>
					<div className="flex items-center gap-4 mb-2">
						{episode.properties.runtimeMinutes && (
							<div className="flex items-center gap-1 text-sm text-zinc-400">
								<Clock className="size-4" />
								{formatRuntime(episode.properties.runtimeMinutes)}
							</div>
						)}
					</div>
					<p className="text-sm text-zinc-300 line-clamp-3">
						{episode.properties.description || "No description available"}
					</p>
				</div>
			</div>
		</div>
	);
};
