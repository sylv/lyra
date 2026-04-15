import { type FC } from "react";
import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../../@generated/gql";
import { getPathForNode } from "../../lib/getPathForMedia";
import { Image, ImageType } from "../image";
import { PlayWrapper } from "../play-wrapper";

interface EpisodePosterDetailProps {
	episode: FragmentType<typeof Fragment>;
}

const Fragment = graphql(`
	fragment EpisodeCard on Node {
		id
		inWatchlist
		unavailableAt
		properties {
			displayName
			description
			thumbnailImage {
				...ImageAsset
			}
			seasonNumber
			episodeNumber
			firstAired
			runtimeMinutes
		}
		watchProgress {
			id
			progressPercent
			completed
			updatedAt
		}
		...GetPathForNode
	}
`);

export const EpisodePosterDetail: FC<EpisodePosterDetailProps> = ({ episode: episodeRef }) => {
	const episode = unmask(Fragment, episodeRef);
	const path = getPathForNode(episode);

	return (
		<div className="flex flex-col gap-2 overflow-hidden select-none">
			<PlayWrapper
				itemId={episode.id}
				path={path}
				unavailable={episode.unavailableAt != null}
				watchProgress={episode.watchProgress}
			>
				<Image
					type={ImageType.Thumbnail}
					asset={episode.properties.thumbnailImage}
					alt={episode.properties.displayName}
					className="w-full"
				/>
			</PlayWrapper>
			<div>
				<Link to={path} className="block min-w-0 text-sm group">
					<span className="text-zinc-300 group-hover:underline">
						S{episode.properties.seasonNumber}E{episode.properties.episodeNumber}{" "}
						<span className="font-semibold">{episode.properties.displayName}</span>
					</span>
				</Link>
				<div className="text-xs text-zinc-400">{formatRuntime(episode.properties.runtimeMinutes)}</div>
			</div>
		</div>
	);
};

const formatRuntime = (minutes: number | null) => {
	if (!minutes) return null;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
};
