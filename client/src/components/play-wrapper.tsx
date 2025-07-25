import type { FC, ReactNode } from "react";
import { FileWarningIcon, PlayIcon } from "lucide-react";
import { setPlayerMedia } from "./player/player-state";
import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { PlayerFrag } from "./player/player-wrapper";

interface PlayWrapperProps {
	media: FragmentOf<typeof PlayWrapperFrag>;
	children: ReactNode;
}

export const PlayWrapperFrag = graphql(
	`
	fragment PlayWrapper on Media {
		...Player
		defaultConnection {
			id
		}
	}
`,
	[PlayerFrag],
);

export const PlayWrapper: FC<PlayWrapperProps> = ({ children, media: mediaRaw }) => {
	const media = readFragment(PlayWrapperFrag, mediaRaw);
	return (
		<div className="relative shrink-0 rounded-lg overflow-hidden group">
			{media.defaultConnection && (
				<button
					type="button"
					className="absolute top-0 left-0 w-full h-full flex items-center justify-center bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity duration-300 cursor-pointer"
					onClick={() => {
						setPlayerMedia(media);
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
