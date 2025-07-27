import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { FileWarningIcon, PlayIcon } from "lucide-react";
import type { FC, ReactNode } from "react";
import { navigate } from "vike/client/router";
import { getPathForMedia, GetPathForMediaFrag } from "../lib/getPathForMedia";
import { setPlayerMedia } from "./player/player-state";
import { PlayerFrag } from "./player/player-wrapper";
import { cn } from "../lib/utils";

interface PlayWrapperProps {
	media: FragmentOf<typeof PlayWrapperFrag>;
	children: ReactNode;
}

export const PlayWrapperFrag = graphql(
	`
	fragment PlayWrapper on Media {
		...Player
		...GetPathForMedia
		defaultConnection {
			id
		}
	}
`,
	[PlayerFrag, GetPathForMediaFrag],
);

export const PlayWrapper: FC<PlayWrapperProps> = ({ children, media: mediaRaw }) => {
	const media = readFragment(PlayWrapperFrag, mediaRaw);
	return (
		<div className="relative shrink rounded-lg overflow-hidden group">
			{media.defaultConnection && (
				<button
					type="button"
					className={cn(
						"absolute top-0 left-0 w-full h-full flex items-center justify-center bg-black/40 opacity-0 cursor-pointer rounded-lg",
						"group-hover:opacity-100 group-hover:border-1 border-white/50 transition-all duration-100",
					)}
					onClick={() => {
						const path = getPathForMedia(media);
						setPlayerMedia(media);
						navigate(path);
					}}
				>
					<PlayIcon className="h-10 w-10 text-white" />
				</button>
			)}
			{!media.defaultConnection && (
				<div className="absolute top-0 left-0 w-full h-full flex items-center justify-center gap-2 p-3 bg-black/60">
					<FileWarningIcon className="h-6 w-6 text-orange-500" />
					<p className="text-sm font-semibold text-orange-100">Unavailable</p>
				</div>
			)}
			{children}
		</div>
	);
};
