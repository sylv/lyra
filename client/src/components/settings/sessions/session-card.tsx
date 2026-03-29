import clsx from "clsx";
import { useRef, type CSSProperties, type FC } from "react";
import { unmask, type FragmentType } from "../../../@generated/gql";
import { formatPlayerTime } from "../../../lib/format-player-time";
import { formatReleaseYear } from "../../../lib/format-release-year";
import { getTimelinePreviewFrameAtMs, sortTimelinePreviewSheets } from "../../../lib/timeline-preview";
import { cn } from "../../../lib/utils";
import { Image, ImageType } from "../../image";
import { PlayerTimelinePreviewSheetFragment } from "../../player/components/player-progress-bar";
import { UserAvatar } from "../../user-avatar";
import { SessionCardFragment } from "./queries";

export const SessionCard: FC<{ session: FragmentType<typeof SessionCardFragment> }> = ({ session: sessionRaw }) => {
	const session = unmask(SessionCardFragment, sessionRaw);
	const node = session.node;
	const file = session.file;
	const previewUrlByAssetId = useRef(new Map<string, string>());
	const poster =
		node?.root?.properties.posterImage ?? node?.properties.posterImage ?? node?.properties.thumbnailImage ?? null;
	const hasEpisodeMetadata =
		!!node?.root?.properties.displayName &&
		node.properties.seasonNumber != null &&
		node.properties.episodeNumber != null;
	const timelinePreviewSheets = sortTimelinePreviewSheets(
		(file?.timelinePreview ?? []).map((sheet) => unmask(PlayerTimelinePreviewSheetFragment, sheet)),
	);
	const previewFrame = getTimelinePreviewFrameAtMs(session.currentPositionMs, timelinePreviewSheets);
	const previewStyle = getPreviewStyle(previewFrame, previewUrlByAssetId.current);
	const releaseYear = node ? formatReleaseYear(node.properties.releasedAt, node.properties.endedAt ?? null) : null;
	const title = hasEpisodeMetadata
		? (node?.root?.properties.displayName ?? "Unavailable item")
		: (node?.properties.displayName ?? "Unavailable item");
	const subtitle = hasEpisodeMetadata
		? `S${node?.properties.seasonNumber}E${node?.properties.episodeNumber} ${node?.properties.displayName}`
		: (releaseYear ?? "Release year unavailable");
	const runtimeSeconds =
		typeof node?.properties.runtimeMinutes === "number" && Number.isFinite(node.properties.runtimeMinutes)
			? node.properties.runtimeMinutes * 60
			: null;
	const positionSeconds = Math.max(0, session.currentPositionMs / 1000);
	const positionLabel =
		runtimeSeconds != null && runtimeSeconds > 0
			? `${formatPlayerTime(positionSeconds)} / ${formatPlayerTime(runtimeSeconds)}`
			: formatPlayerTime(positionSeconds);
	const playerCountLabel = `${session.players.length} ${session.players.length === 1 ? "viewer" : "viewers"}`;

	return (
		<article className="flex flex-col rounded-md overflow-hidden bg-zinc-950/30">
			<div className="space-y-3">
				<div className="flex select-none">
					<Image type={ImageType.Poster} asset={poster} alt={title} className="h-32 w-fit" />
					<div
						className={clsx(
							"overflow-hidden bg-black/80 h-32 aspect-video",
							!previewStyle && "flex items-center justify-center text-zinc-600 text-xs",
						)}
					>
						{previewStyle && <div className="w-full bg-black" style={previewStyle} />}
						{!previewStyle && (
							<div className="flex flex-col items-center gap-2">
								<span>(ノಠ益ಠ)ノ彡┻━┻</span>
								<p>No preview available</p>
							</div>
						)}
					</div>
				</div>
			</div>
			<div className="mt-auto space-y-3 p-4">
				<div className="space-y-1">
					<h3 className="line-clamp-2 text-sm font-medium text-zinc-100">{title}</h3>
					<p className="line-clamp-2 text-xs text-zinc-400">{subtitle}</p>
					<p className="text-xs text-zinc-500">{positionLabel}</p>
				</div>
				<div className="space-y-2">
					{session.players.map((player) => {
						const username = player.user?.username ?? "Unknown user";
						return (
							<div key={player.id} className="flex items-center gap-3 rounded-lg py-2">
								{player.user ? (
									<UserAvatar createdAt={player.user.createdAt} alt="" className="size-7" size={32} />
								) : (
									<div className="size-7 rounded-full bg-zinc-800" />
								)}
								<div className="min-w-0">
									<p className={cn("truncate text-sm text-zinc-100")}>{username}</p>
								</div>
							</div>
						);
					})}
				</div>
			</div>
		</article>
	);
};

const getPreviewStyle = (
	frame: ReturnType<typeof getTimelinePreviewFrameAtMs>,
	previewUrlByAssetId: Map<string, string>,
): CSSProperties | undefined => {
	if (!frame) return undefined;

	const assetKey = frame.assetId ?? frame.assetSignedUrl;
	const stableSignedUrl = previewUrlByAssetId.get(assetKey) ?? frame.assetSignedUrl;
	if (!previewUrlByAssetId.has(assetKey)) {
		previewUrlByAssetId.set(assetKey, frame.assetSignedUrl);
	}

	const widthPercent = (frame.sheetWidthPx / frame.frameWidthPx) * 100;
	const heightPercent = (frame.sheetHeightPx / frame.frameHeightPx) * 100;
	const positionXPercent =
		frame.sheetWidthPx === frame.frameWidthPx ? 0 : (frame.offsetXPx / (frame.sheetWidthPx - frame.frameWidthPx)) * 100;
	const positionYPercent =
		frame.sheetHeightPx === frame.frameHeightPx
			? 0
			: (frame.offsetYPx / (frame.sheetHeightPx - frame.frameHeightPx)) * 100;

	return {
		aspectRatio: `${frame.frameWidthPx} / ${frame.frameHeightPx}`,
		backgroundImage: `url(${stableSignedUrl})`,
		backgroundPosition: `${positionXPercent}% ${positionYPercent}%`,
		backgroundRepeat: "no-repeat",
		backgroundSize: `${widthPercent}% ${heightPercent}%`,
	};
};
