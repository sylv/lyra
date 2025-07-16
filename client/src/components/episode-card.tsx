import type { FC } from "react";
import type { MediaWithFirstConnection } from "../@generated/server";
import { PlayWrapper } from "./play-wrapper";
import { PlayIcon, Clock } from "lucide-react";
import { Thumbnail } from "./thumbnail";

interface EpisodeCardProps {
	episode: MediaWithFirstConnection;
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

export const EpisodeCard: FC<EpisodeCardProps> = ({
	episode,
	showSeasonInfo = false,
}) => {
	return (
		<div className="group flex gap-4 p-4 hover:bg-zinc-800/10 rounded-lg transition-colors border border-zinc-800/50">
			<div className="relative flex-shrink-0 rounded-md overflow-hidden">
				<PlayWrapper media={episode}>
					<Thumbnail
						imageUrl={episode.media.thumbnail_url}
						alt={episode.media.name}
						className="aspect-[16/9] h-36 object-cover"
					/>
				</PlayWrapper>
			</div>
			<div className="flex-1 min-w-0">
				<h3 className="font-semibold text-white mb-1">
					<span className="text-zinc-400 text-sm font-normal mr-2">
						{showSeasonInfo ? `S${episode.media.season_number}` : ""}E
						{episode.media.episode_number}
					</span>
					{episode.media.name}
				</h3>
				{episode.media.runtime_minutes && (
					<div className="flex items-center gap-1 text-sm text-zinc-400 mb-2">
						<Clock className="w-4 h-4" />
						{formatRuntime(episode.media.runtime_minutes)}
					</div>
				)}
				<p className="text-sm text-zinc-300 line-clamp-3">
					{episode.media.description || "No description available"}
				</p>
			</div>
		</div>
	);
};
