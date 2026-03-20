import { useNavigate } from "@tanstack/react-router";
import { useMemo, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { getPathForNode } from "../lib/getPathForMedia";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { openPlayerMedia } from "./player/player-state";

interface EpisodeCardProps {
	episode: FragmentType<typeof Fragment>;
}

const formatRuntime = (minutes: number | null) => {
	if (!minutes) return null;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
};

const Fragment = graphql(`
	fragment EpisodeCard on Node {
		id
		name
		properties {
			description
			thumbnailImage {
				...ImageAsset
			}
			seasonNumber
			episodeNumber
			releasedAt
			runtimeMinutes
		}
		watchProgress {
			progressPercent
			completed
			updatedAt
		}
		...GetPathForNode
	}
`);

export const EpisodeCard: FC<EpisodeCardProps> = ({ episode: episodeRef }) => {
	const episode = unmask(Fragment, episodeRef);
	const navigate = useNavigate();
	const path = getPathForNode(episode);
	const releaseDate = useMemo(() => {
		if (!episode.properties.releasedAt) return null;
		return new Date(episode.properties.releasedAt * 1000).toLocaleDateString(undefined, {
			year: "numeric",
			month: "short",
			day: "numeric",
		});
	}, [episode.properties.releasedAt]);

	return (
		<button
			type="button"
			className="group flex gap-4 group/play w-full text-left"
			aria-label={`Play ${episode.name}`}
			onClick={() => {
				openPlayerMedia(episode.id, true);
				navigate({ to: path as never });
			}}
		>
			<div className="relative overflow-hidden h-min rounded-sm shrink-0">
				<PlayWrapper itemId={episode.id} path={path} watchProgress={episode.watchProgress}>
					<Image
						type={ImageType.Thumbnail}
						asset={episode.properties.thumbnailImage}
						alt={episode.name}
						className="h-36"
					/>
				</PlayWrapper>
			</div>
			<div>
				<h3 className="font-semibold text-white flex gap-3">
					<div className="text-zinc-300">
						S{episode.properties.seasonNumber}E{episode.properties.episodeNumber}
					</div>
					{episode.name}
				</h3>
				<div className="flex items-center gap-3 text-zinc-400 mb-2 text-sm">
					{releaseDate && <div>{releaseDate}</div>}
					{episode.properties.runtimeMinutes && <div>{formatRuntime(episode.properties.runtimeMinutes)}</div>}
				</div>
				<p className="text-xs text-zinc-300 line-clamp-3">
					{episode.properties.description || "No description available"}
				</p>
			</div>
		</button>
	);
};
