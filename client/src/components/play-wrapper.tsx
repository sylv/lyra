import { CheckCheckIcon, FileWarningIcon, PlayIcon } from "lucide-react";
import { Fragment, type FC, type ReactNode } from "react";
import { useNavigate } from "react-router";
import { cn } from "../lib/utils";
import { openPlayerMedia } from "./player/player-context";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface PlayWrapperProps {
	itemId?: string | null;
	path: string;
	unavailable?: boolean | null;
	watchProgress?: {
		completed?: boolean;
		progressPercent: number;
		updatedAt: number;
	} | null;
	children: ReactNode;
}

export const PlayWrapper: FC<PlayWrapperProps> = ({ children, path, itemId, unavailable, watchProgress }) => {
	const navigate = useNavigate();
	const playableItemId = unavailable ? null : itemId;

	return (
		<div className="group/play relative inline-block overflow-hidden rounded-sm">
			{playableItemId && (
				<button
					type="button"
					className={cn(
						"absolute left-0 top-0 z-10 flex h-full w-full cursor-pointer items-center justify-center bg-black/40 opacity-0",
						"transition-opacity duration-75 group-hover/play:opacity-100",
					)}
					onClick={() => {
						openPlayerMedia(playableItemId, true);
						navigate(path);
					}}
				>
					<PlayIcon className="h-10 w-10 text-white" />
				</button>
			)}
			{watchProgress && !watchProgress.completed && (
				<Fragment>
					<div
						className="absolute bottom-0 left-0 z-10 h-1 bg-white/80"
						style={{ width: `${watchProgress.progressPercent * 100}%` }}
					/>
					<div className="absolute bottom-0 left-0 right-0 z-10 h-1 bg-white/20" />
				</Fragment>
			)}
			{watchProgress && watchProgress.completed && (
				<UnplayedItemsTab>
					<CheckCheckIcon className="size-4" strokeWidth={2.5} />
				</UnplayedItemsTab>
			)}
			{unavailable && (
				<div className="absolute left-0 top-0 flex h-full w-full select-none items-center justify-center gap-2 bg-black/60 p-3">
					<FileWarningIcon className="h-6 w-6 text-orange-500" />
					<p className="text-sm font-semibold text-orange-100">Unavailable</p>
				</div>
			)}
			{children}
		</div>
	);
};
