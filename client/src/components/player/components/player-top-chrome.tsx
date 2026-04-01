import { ChevronDown, LoaderCircle, UserX2, Users, XIcon } from "lucide-react";
import type { FC } from "react";
import { useNavigate } from "react-router";
import { useMutation } from "urql";
import { unmask } from "../../../@generated/gql";
import { type ItemPlaybackQuery, WatchSessionActionKind } from "../../../@generated/gql/graphql";
import { getPathForNodeData } from "../../../lib/getPathForMedia";
import { cn } from "../../../lib/utils";
import { clearPlayerMedia, setPlayerControls, togglePlayerFullscreen, usePlayerContext } from "../player-context";
import { usePlayerRefsContext } from "../player-refs-context";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "../../ui/dropdown-menu";
import { PlayerButton } from "./player-button";
import { formatReleaseYear } from "../../../lib/format-release-year";
import { WatchSessionAction, WatchSessionBeaconFragment } from "../player-queries";
import { applyWatchSessionBeacon } from "../watch-session";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const PlayerTopChrome: FC<{ media: CurrentMedia }> = ({ media }) => {
	const showControls = usePlayerContext((ctx) => ctx.controls.showControls);
	const isFullscreen = usePlayerContext((ctx) => ctx.state.isFullscreen);
	const isWatchSessionMenuOpen = usePlayerContext((ctx) => ctx.controls.isWatchSessionMenuOpen);
	const watchSession = usePlayerContext((ctx) => ctx.watchSession);
	const { containerRef } = usePlayerRefsContext();
	const navigate = useNavigate();
	const [{ fetching: removingPlayer }, watchSessionAction] = useMutation(WatchSessionAction);
	const detailsPath = media.libraryId ? getPathForNodeData(media) : null;
	const hasEpisodeMetadata =
		!!media.root?.properties.displayName &&
		media.properties.seasonNumber != null &&
		media.properties.episodeNumber != null;
	const shareUrl = watchSession.sessionId
		? `${window.location.origin}/?watchSession=${encodeURIComponent(watchSession.sessionId)}`
		: null;

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
						navigate(detailsPath);
					}}
				>
					{hasEpisodeMetadata ? (
						<>
							<h2 className={cn("font-semibold", isFullscreen ? "text-xl" : "text-sm")}>
								{media.root?.properties.displayName}
							</h2>
							<p className={cn("text-gray-300", isFullscreen ? "text-sm" : "text-xs")}>
								S{media.properties.seasonNumber}E{media.properties.episodeNumber} {media.properties.displayName}
							</p>
						</>
					) : (
						<>
							<h2 className={cn("font-semibold", isFullscreen ? "text-xl" : "text-sm")}>
								{media.properties.displayName}
							</h2>
							<p className={cn("text-gray-300", isFullscreen ? "text-sm" : "text-xs")}>
								{formatReleaseYear(media.properties.releasedAt, media.properties.endedAt)}
							</p>
						</>
					)}
				</button>
			</div>
			<div className="pointer-events-auto flex items-center gap-3 text-white">
				{watchSession.sessionId ? (
					<DropdownMenu
						open={isWatchSessionMenuOpen}
						onOpenChange={(open) => setPlayerControls({ isWatchSessionMenuOpen: open })}
					>
						<DropdownMenuTrigger asChild>
							<PlayerButton
								aria-label="Open watch session menu"
								onClick={(event) => {
									event.stopPropagation();
								}}
							>
								<Users className={isFullscreen ? "size-6" : "size-5"} />
							</PlayerButton>
						</DropdownMenuTrigger>
						<DropdownMenuContent
							align="end"
							portalContainer={containerRef.current}
							onClick={(event) => event.stopPropagation()}
							className="z-[80] w-80 border-zinc-700 bg-black/95 p-3 text-zinc-100 shadow-xl shadow-black/40"
						>
							<div className="space-y-3">
								<div className="space-y-1">
									<p className="text-sm text-zinc-300">
										{watchSession.mode === "SYNCED" ? "Synced" : "Advisory"} · {watchSession.effectiveState}
									</p>
									{watchSession.connectionWarning ? (
										<p className="text-xs text-orange-300">{watchSession.connectionWarning}</p>
									) : null}
								</div>
								{shareUrl ? (
									<div className="space-y-1">
										<p className="text-xs uppercase font-semibold text-zinc-500">Invite Link</p>
										<input
											readOnly
											value={shareUrl}
											onFocus={(event) => event.currentTarget.select()}
											className="w-full rounded border border-zinc-800 bg-zinc-950 px-3 py-2 text-xs text-zinc-200 outline-none"
										/>
									</div>
								) : null}
								<div className="space-y-1">
									<p className="text-xs uppercase font-semibold text-zinc-500">Players</p>
									<div className="space-y-1">
										{watchSession.players.map((player) => (
											<div
												key={player.id}
												className="flex items-center justify-between rounded border border-zinc-800/80 bg-zinc-950/80 px-3 py-2"
											>
												<div>
													<p className="text-sm">
														{player.username}
														{player.id === watchSession.playerId ? " (this device)" : ""}
													</p>
													<p className="text-xs text-zinc-500">
														{player.isInactive ? "Inactive" : player.isBuffering ? "Buffering" : "Connected"}
													</p>
												</div>
												{player.canRemove ? (
													<button
														type="button"
														className="rounded p-1 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
														aria-label={`Remove ${player.username}`}
														onClick={() => {
															if (!watchSession.sessionId || !watchSession.playerId) return;
															void watchSessionAction({
																input: {
																	sessionId: watchSession.sessionId,
																	playerId: watchSession.playerId,
																	kind: WatchSessionActionKind.RemovePlayer,
																	positionMs: null,
																	nodeId: null,
																	targetPlayerId: player.id,
																},
															}).then((result) => {
																if (result.error) {
																	throw result.error;
																}
																const beacon = result.data?.watchSessionAction;
																if (beacon) {
																	applyWatchSessionBeacon(unmask(WatchSessionBeaconFragment, beacon));
																}
															});
														}}
													>
														{removingPlayer ? (
															<LoaderCircle className="size-4 animate-spin" />
														) : (
															<UserX2 className="size-4" />
														)}
													</button>
												) : null}
											</div>
										))}
									</div>
								</div>
							</div>
						</DropdownMenuContent>
					</DropdownMenu>
				) : null}
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
