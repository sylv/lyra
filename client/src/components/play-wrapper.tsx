import { FileWarningIcon, PlayIcon } from "lucide-react";
import { Fragment, type FC, type ReactNode } from "react";
import { navigate } from "vike/client/router";
import { cn } from "../lib/utils";
import { setPlayerMedia } from "./player/player-state";

interface PlayWrapperProps {
	itemId?: string | null;
	path: string;
	watchProgress?: {
		progressPercent: number;
		updatedAt: number;
	} | null;
	children: ReactNode;
}

export const PlayWrapper: FC<PlayWrapperProps> = ({ children, path, itemId, watchProgress }) => {
	return (
		<div className="relative shrink-0 overflow-hidden group/play rounded-sm">
			{itemId && (
				<button
					type="button"
					className={cn(
						"absolute top-0 left-0 w-full h-full flex items-center justify-center bg-black/20 opacity-0 cursor-pointer",
						"group-hover/play:opacity-100 group-hover/play:border border-white/50",
					)}
					onClick={() => {
						if (!itemId) return;
						setPlayerMedia(itemId, true);
						navigate(path);
					}}
				>
					<PlayIcon className="h-10 w-10 text-white" />
				</button>
			)}
			{watchProgress && (
				<Fragment>
					<div
						className="z-10 absolute bottom-0 left-0 bg-white/80 h-1"
						style={{
							width: `${watchProgress.progressPercent * 100}%`,
						}}
					/>
					<div className="z-10 absolute bottom-0 left-0 right-0 bg-white/20 h-1" />
				</Fragment>
			)}
			{!itemId && (
				<div className="absolute top-0 left-0 w-full h-full flex items-center justify-center gap-2 p-3 bg-black/60 select-none">
					<FileWarningIcon className="h-6 w-6 text-orange-500" />
					<p className="text-sm font-semibold text-orange-100">Unavailable</p>
				</div>
			)}
			{children}
		</div>
	);
};
