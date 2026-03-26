import { useNavigate } from "@tanstack/react-router";
import { ChevronDown, XIcon } from "lucide-react";
import type { FC } from "react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { getPathForNodeData } from "../../../lib/getPathForMedia";
import { cn } from "../../../lib/utils";
import { clearPlayerMedia, togglePlayerFullscreen, usePlayerContext } from "../player-context";
import { PlayerButton } from "./player-button";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const PlayerTopChrome: FC<{ media: CurrentMedia }> = ({ media }) => {
	const showControls = usePlayerContext((ctx) => ctx.controls.showControls);
	const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
	const navigate = useNavigate();
	const detailsPath = media.libraryId ? getPathForNodeData(media) : null;
	const hasEpisodeMetadata =
		!!media.root?.properties.displayName && media.properties.seasonNumber != null && media.properties.episodeNumber != null;

	return (
		<div
			className={cn(
				"pointer-events-none flex items-center justify-between transition-opacity duration-300",
				showControls ? "opacity-100" : "opacity-0",
				isFullscreen ? "p-6" : "p-3",
			)}
		>
			<div className="pointer-events-auto flex items-center gap-3 text-white">
				{isFullscreen && (
					<PlayerButton
						aria-label="Go back"
						onClick={(event) => {
							event.stopPropagation();
							togglePlayerFullscreen(false);
						}}
					>
						<ChevronDown className="size-6" />
					</PlayerButton>
				)}
				<button
					type="button"
					className={cn("rounded-sm text-left transition-colors", detailsPath ? "cursor-pointer" : "cursor-default")}
					onClick={(event) => {
						event.stopPropagation();
						if (!detailsPath) return;
						togglePlayerFullscreen(false);
						navigate({ to: detailsPath });
					}}
				>
					{hasEpisodeMetadata ? (
						<>
							<h2 className={cn("font-semibold", isFullscreen ? "text-xl" : "text-sm")}>
								{media.root?.properties.displayName}: Season {media.properties.seasonNumber}
							</h2>
							<p className={cn("text-gray-300", isFullscreen ? "text-sm" : "text-xs")}>
								Episode {media.properties.episodeNumber}: {media.properties.displayName}
							</p>
						</>
					) : (
						<h2 className={cn("font-semibold", isFullscreen ? "text-xl" : "text-sm")}>{media.properties.displayName}</h2>
					)}
				</button>
			</div>
			<div className="pointer-events-auto flex items-center gap-3 text-white">
				<PlayerButton
					aria-label="Close player"
					onClick={(event) => {
						event.stopPropagation();
						clearPlayerMedia();
					}}
				>
					<XIcon className={isFullscreen ? "size-6" : "size-5"} />
				</PlayerButton>
			</div>
		</div>
	);
};
