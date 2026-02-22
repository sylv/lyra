import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { Clock } from "lucide-react";
import type { FC } from "react";
import { getPathForItem, GetPathForItemFrag } from "../lib/getPathForMedia";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { setPlayerMedia } from "./player/player-state";
import { navigate } from "vike/client/router";

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
		<button
			type="button"
			className="group flex gap-4 group/play w-full text-left"
			onClick={() => {
				if (!episode.id) return;
				setPlayerMedia(episode.id, true);
				navigate(path);
			}}
		>
			<div className="relative overflow-hidden h-min rounded-sm">
				<PlayWrapper itemId={episode.id} path={path} watchProgress={episode.watchProgress}>
					<Image
						type={ImageType.Thumbnail}
						imageUrl={episode.properties.thumbnailUrl}
						alt={episode.name}
						className="h-30"
					/>
				</PlayWrapper>
			</div>
			<div>
				<h3 className="font-semibold text-white">{episode.name}</h3>
				<div className="flex items-center gap-4 text-zinc-400 mb-2 text-sm">
					<div>
						S{episode.properties.seasonNumber}E{episode.properties.episodeNumber}
					</div>
					{episode.properties.runtimeMinutes && <div>{formatRuntime(episode.properties.runtimeMinutes)}</div>}
				</div>
				<p className="text-xs text-zinc-300 line-clamp-3">
					{episode.properties.description || "No description available"}
				</p>
			</div>
		</button>
	);
};
