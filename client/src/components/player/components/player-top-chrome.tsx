import { useNavigate } from "@tanstack/react-router";
import { ChevronDown, XIcon } from "lucide-react";
import type { FC } from "react";
import { useStore } from "zustand/react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { getPathForNodeData } from "../../../lib/getPathForMedia";
import { cn } from "../../../lib/utils";
import { clearPlayerMedia, playerState, togglePlayerFullscreen } from "../player-state";
import { videoState } from "../video-state";
import { PlayerButton } from "./player-button";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

interface PlayerTopChromeProps {
	media: CurrentMedia;
}

export const PlayerTopChrome: FC<PlayerTopChromeProps> = ({ media }) => {
	const showControls = useStore(videoState, (s) => s.showControls);
	const isFullscreen = useStore(playerState, (s) => s.isFullscreen);
	const navigate = useNavigate();

	const detailsPath = media.libraryId ? getPathForNodeData(media) : null;

	return (
		<div
			className={cn(
				"flex justify-between items-center transition-opacity duration-300 pointer-events-none",
				showControls ? "opacity-100" : "opacity-0",
				isFullscreen ? "p-6" : "p-4",
			)}
		>
			<div className="flex items-center gap-3 text-white pointer-events-auto">
				{isFullscreen && (
					<PlayerButton
						aria-label="Go back"
						onClick={(e) => {
							e.stopPropagation();
							togglePlayerFullscreen(false);
						}}
					>
						<ChevronDown className="size-6" />
					</PlayerButton>
				)}
				{media.root?.properties.displayName && media.properties.seasonNumber && media.properties.episodeNumber ? (
					<button
						type="button"
						className={cn(
							"text-left rounded-sm transition-colors",
							detailsPath ? "cursor-pointer group" : "cursor-default",
						)}
						onClick={(event) => {
							event.stopPropagation();
							if (detailsPath) {
								togglePlayerFullscreen(false);
								navigate({ to: detailsPath });
							}
						}}
					>
						<h2 className="text-xl font-semibold group-hover:underline">
							{media.root.properties.displayName}: Season {media.properties.seasonNumber}
						</h2>
						<p className="text-sm text-gray-300">
							Episode {media.properties.episodeNumber}: {media.properties.displayName}
						</p>
					</button>
				) : (
					<button
						type="button"
						className={cn(
							"text-left rounded-sm transition-colors",
							detailsPath ? "cursor-pointer hover:underline" : "cursor-default",
						)}
						onClick={(event) => {
							event.stopPropagation();
							if (detailsPath) {
								togglePlayerFullscreen(false);
								navigate({ to: detailsPath });
							}
						}}
					>
						<h2 className="text-xl font-semibold">{media.properties.displayName}</h2>
					</button>
				)}
			</div>
			<div className="flex items-center gap-3 text-white pointer-events-auto">
				<PlayerButton
					aria-label="Close player"
					onClick={(event) => {
						event.stopPropagation();
						clearPlayerMedia();
					}}
				>
					<XIcon className="size-6" />
				</PlayerButton>
			</div>
		</div>
	);
};
