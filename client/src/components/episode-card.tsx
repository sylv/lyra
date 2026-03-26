import { useNavigate } from "@tanstack/react-router";
import { useMemo, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { getPathForNode } from "../lib/getPathForMedia";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { openPlayerMedia } from "./player/player-context";

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
		properties {
			displayName
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
			className="group/play flex w-full gap-4 text-left"
			aria-label={`Play ${episode.properties.displayName}`}
			onClick={() => {
				openPlayerMedia(episode.id, true);
				navigate({ to: path });
			}}
		>
			<div className="relative h-min shrink-0 overflow-hidden rounded-sm">
				<PlayWrapper itemId={episode.id} path={path} watchProgress={episode.watchProgress}>
					<Image
						type={ImageType.Thumbnail}
						asset={episode.properties.thumbnailImage}
						alt={episode.properties.displayName}
						className="aspect-[12:8] h-20 md:h-36"
					/>
				</PlayWrapper>
			</div>
			<div>
				<h3 className="flex gap-3 font-semibold text-white">
					<div className="text-zinc-300">
						S{episode.properties.seasonNumber}E{episode.properties.episodeNumber}
					</div>
					{episode.properties.displayName}
				</h3>
				<div className="mb-2 flex items-center gap-3 text-sm text-zinc-400">
					{releaseDate && <div>{releaseDate}</div>}
					{episode.properties.runtimeMinutes && <div>{formatRuntime(episode.properties.runtimeMinutes)}</div>}
				</div>
				<p className="line-clamp-3 text-xs text-zinc-300">{episode.properties.description || "No description available"}</p>
			</div>
		</button>
	);
};
