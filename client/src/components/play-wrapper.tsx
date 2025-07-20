import type { FC, ReactNode } from "react";
import type { MediaWithFirstConnection } from "../@generated/server";
import { FileWarningIcon, PlayIcon } from "lucide-react";
import { setPlayerMedia } from "./player/player-state";

interface PlayWrapperProps {
	media: MediaWithFirstConnection;
	children: ReactNode;
}

export const PlayWrapper: FC<PlayWrapperProps> = ({ children, media }) => {
	return (
		<div className="relative shrink-0 rounded-lg overflow-hidden group">
			{media.default_connection && (
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
			{!media.default_connection && (
				<div className="absolute top-0 left-0 w-full h-full flex items-center justify-center gap-2 p-3 bg-black/60">
					<FileWarningIcon className="h-6 w-6 text-orange-500" />
					<p className="text-sm font-semibold text-orange-100">Unavailable</p>
				</div>
			)}
			{children}
		</div>
	);
};
