import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import { useMemo, useState, type FC } from "react";
import { formatPlayerTime } from "../../../lib/format-player-time";
import { cn } from "../../../lib/utils";

const TIMELINE_PREVIEW_THUMBNAIL_WIDTH_PX = 380;
const TIMELINE_TIME_TOOLTIP_WIDTH_PX = 56;

export const PlayerTimelinePreviewSheetFragment = graphql(`
	fragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {
		positionMs
		endMs
		sheetIntervalMs
		sheetGapSize
		asset {
			id
			signedUrl
			width
			height
		}
	}
`);

interface PlayerProcessBarProps {
	duration: number;
	currentTime: number;
	bufferedRanges: { start: number; end: number }[];
	timelinePreviewSheets: FragmentType<typeof PlayerTimelinePreviewSheetFragment>[];
	onChange: (time: number) => void;
	onInteractionStart: () => void;
	onInteractionEnd: () => void;
	onActivity: () => void;
}

interface HoverPreviewFrame {
	assetSignedUrl: string;
	sheetWidthPx: number;
	sheetHeightPx: number;
	frameWidthPx: number;
	frameHeightPx: number;
	offsetXPx: number;
	offsetYPx: number;
}

const getHoverPreviewFrame = (
	hoverTimeSeconds: number,
	sheets: Array<FragmentType<typeof PlayerTimelinePreviewSheetFragment>>,
): HoverPreviewFrame | null => {
	if (!Number.isFinite(hoverTimeSeconds) || hoverTimeSeconds < 0) {
		return null;
	}

	const hoverMs = hoverTimeSeconds * 1000;

	for (const sheetRef of sheets) {
		const sheet = unmask(PlayerTimelinePreviewSheetFragment, sheetRef);
		const sheetWidthPx = sheet.asset.width ?? 0;
		const sheetHeightPx = sheet.asset.height ?? 0;
		if (sheetWidthPx <= 0 || sheetHeightPx <= 0 || sheet.sheetGapSize < 0 || sheet.sheetIntervalMs <= 0) {
			continue;
		}

		const frameCount = Math.floor((sheet.endMs - sheet.positionMs) / sheet.sheetIntervalMs);
		if (frameCount <= 0) {
			continue;
		}

		// Timeline preview frames are offset by one interval (first frame is at +interval, not 0s).
		// Use nearest-frame selection to avoid previews feeling delayed.
		const previewTimestampMs = Math.max(
			sheet.sheetIntervalMs,
			Math.round(hoverMs / sheet.sheetIntervalMs) * sheet.sheetIntervalMs,
		);
		if (previewTimestampMs <= sheet.positionMs || previewTimestampMs > sheet.endMs) {
			continue;
		}

		const columns = Math.max(1, Math.ceil(Math.sqrt(frameCount)));
		const rows = Math.max(1, Math.ceil(frameCount / columns));
		const frameWidthPx = Math.floor((sheetWidthPx - (columns + 1) * sheet.sheetGapSize) / columns);
		const frameHeightPx = Math.floor((sheetHeightPx - (rows + 1) * sheet.sheetGapSize) / rows);
		if (frameWidthPx <= 0 || frameHeightPx <= 0) {
			continue;
		}

		const rawIndex = Math.floor((previewTimestampMs - sheet.positionMs) / sheet.sheetIntervalMs) - 1;
		const frameIndex = Math.max(0, Math.min(frameCount - 1, rawIndex));
		const columnIndex = frameIndex % columns;
		const rowIndex = Math.floor(frameIndex / columns);
		const offsetXPx = sheet.sheetGapSize + columnIndex * (frameWidthPx + sheet.sheetGapSize);
		const offsetYPx = sheet.sheetGapSize + rowIndex * (frameHeightPx + sheet.sheetGapSize);

		return {
			assetSignedUrl: sheet.asset.signedUrl,
			sheetWidthPx,
			sheetHeightPx,
			frameWidthPx,
			frameHeightPx,
			offsetXPx,
			offsetYPx,
		};
	}

	return null;
};

export const PlayerProgressBar: FC<PlayerProcessBarProps> = ({
	duration,
	currentTime,
	bufferedRanges,
	timelinePreviewSheets,
	onChange,
	onInteractionStart,
	onInteractionEnd,
	onActivity,
}) => {
	const [hoverState, setHoverState] = useState<{
		time: number;
		xPx: number;
		barWidthPx: number;
	} | null>(null);
	const sortedTimelinePreviewSheets = useMemo(() => {
		return [...timelinePreviewSheets].sort((aRef, bRef) => {
			const a = unmask(PlayerTimelinePreviewSheetFragment, aRef);
			const b = unmask(PlayerTimelinePreviewSheetFragment, bRef);
			if (a.positionMs !== b.positionMs) {
				return a.positionMs - b.positionMs;
			}
			return a.asset.id.localeCompare(b.asset.id);
		});
	}, [timelinePreviewSheets]);
	const hoverPreviewFrame = useMemo(() => {
		if (hoverState == null) {
			return null;
		}
		return getHoverPreviewFrame(hoverState.time, sortedTimelinePreviewSheets);
	}, [hoverState, sortedTimelinePreviewSheets]);
	const renderedHoverPreviewFrame = useMemo(() => {
		if (hoverPreviewFrame == null) {
			return null;
		}

		const scale = TIMELINE_PREVIEW_THUMBNAIL_WIDTH_PX / hoverPreviewFrame.frameWidthPx;
		return {
			assetSignedUrl: hoverPreviewFrame.assetSignedUrl,
			frameWidthPx: Math.max(1, TIMELINE_PREVIEW_THUMBNAIL_WIDTH_PX),
			frameHeightPx: Math.max(1, hoverPreviewFrame.frameHeightPx * scale),
			scale,
			offsetXPx: hoverPreviewFrame.offsetXPx,
			offsetYPx: hoverPreviewFrame.offsetYPx,
			sheetWidthPx: hoverPreviewFrame.sheetWidthPx,
			sheetHeightPx: hoverPreviewFrame.sheetHeightPx,
			sourceFrameWidthPx: hoverPreviewFrame.frameWidthPx,
			sourceFrameHeightPx: hoverPreviewFrame.frameHeightPx,
		};
	}, [hoverPreviewFrame]);
	const hoverMarkerPercent = useMemo(() => {
		if (hoverState == null || hoverState.barWidthPx <= 0) {
			return 0;
		}

		return (hoverState.xPx / hoverState.barWidthPx) * 100;
	}, [hoverState]);
	const clampedHoverOverlayPercent = useMemo(() => {
		if (hoverState == null || hoverState.barWidthPx <= 0) {
			return 0;
		}

		const overlayWidthPx = renderedHoverPreviewFrame?.frameWidthPx ?? TIMELINE_TIME_TOOLTIP_WIDTH_PX;
		const minCenterPx = overlayWidthPx / 2;
		const maxCenterPx = hoverState.barWidthPx - overlayWidthPx / 2;
		const clampedCenterPx =
			minCenterPx <= maxCenterPx
				? Math.min(Math.max(hoverState.xPx, minCenterPx), maxCenterPx)
				: hoverState.barWidthPx / 2;

		return (clampedCenterPx / hoverState.barWidthPx) * 100;
	}, [hoverState, renderedHoverPreviewFrame]);

	const handleProgressClick = (event: React.MouseEvent<HTMLDivElement>) => {
		event.stopPropagation();
		onActivity();

		const rect = event.currentTarget.getBoundingClientRect();
		const clickX = event.clientX - rect.left;
		const ratio = Math.max(0, Math.min(1, clickX / rect.width));
		const newTime = ratio * duration;
		onChange(newTime);
	};

	const handleProgressMouseMove = (event: React.MouseEvent<HTMLDivElement>) => {
		if (!duration) return;
		onActivity();

		const rect = event.currentTarget.getBoundingClientRect();
		const hoverX = event.clientX - rect.left;
		const ratio = Math.max(0, Math.min(1, hoverX / rect.width));
		const hoverTimeValue = ratio * duration;
		setHoverState({
			time: Math.max(0, Math.min(duration, hoverTimeValue)),
			xPx: Math.max(0, Math.min(rect.width, hoverX)),
			barWidthPx: rect.width,
		});
	};

	const onMouseLeave = () => {
		setHoverState(null);
	};

	const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
		if (!duration) {
			return;
		}

		const step = 5;
		if (event.key === "ArrowLeft") {
			event.preventDefault();
			onActivity();
			onChange(Math.max(0, currentTime - step));
			return;
		}
		if (event.key === "ArrowRight") {
			event.preventDefault();
			onActivity();
			onChange(Math.min(duration, currentTime + step));
			return;
		}
		if (event.key === "Home") {
			event.preventDefault();
			onActivity();
			onChange(0);
			return;
		}
		if (event.key === "End") {
			event.preventDefault();
			onActivity();
			onChange(duration);
		}
	};

	const progressPercent = duration ? (currentTime / duration) * 100 : 0;

	return (
		<div
			className="py-2 my-2 cursor-pointer"
			onClick={handleProgressClick}
			onMouseMove={handleProgressMouseMove}
			onMouseLeave={onMouseLeave}
			onKeyDown={handleKeyDown}
			onPointerDown={onInteractionStart}
			onPointerUp={onInteractionEnd}
			onPointerCancel={onInteractionEnd}
			role="slider"
			tabIndex={0}
			aria-label="Seek video"
			aria-valuemin={0}
			aria-valuemax={duration || 100}
			aria-valuenow={currentTime || 0}
		>
			<div className="relative h-1 bg-white/15 group-hover:h-2 transition-all rounded-md">
				<div className="h-full bg-white/80 transition-all rounded-md" style={{ width: `${progressPercent}%` }} />
				{/* Buffered ranges */}
				{bufferedRanges.map((range) => {
					if (!duration) return null;
					const startPercent = (range.start / duration) * 100;
					const widthPercent = ((range.end - range.start) / duration) * 100;
					return (
						<div
							key={`${range.start}-${range.end}`}
							className="h-full absolute top-0 bg-white/15 transition-all"
							style={{
								left: `${startPercent}%`,
								width: `${widthPercent}%`,
							}}
						/>
					);
				})}
				{/* Hover time tooltip */}
				{hoverState != null && (
					<>
						<div
							className="absolute top-0 bottom-0 pointer-events-none"
							style={{
								left: `${hoverMarkerPercent}%`,
							}}
						>
							<div className={cn("absolute -top-1 bottom-0 w-0.5 shadow-lg z-20 bg-white/40 -translate-x-1/2")} />
						</div>
						<div
							className="absolute top-0 bottom-0 pointer-events-none"
							style={{
								left: `${clampedHoverOverlayPercent}%`,
							}}
						>
							{renderedHoverPreviewFrame && (
								<div
									className="absolute left-1/2 -translate-x-1/2 bottom-4 rounded-md bg-black shadow-lg overflow-hidden"
									style={{
										width: `${renderedHoverPreviewFrame.frameWidthPx}px`,
										height: `${renderedHoverPreviewFrame.frameHeightPx}px`,
									}}
								>
									<div
										style={{
											width: `${renderedHoverPreviewFrame.sourceFrameWidthPx}px`,
											height: `${renderedHoverPreviewFrame.sourceFrameHeightPx}px`,
											transform: `scale(${renderedHoverPreviewFrame.scale})`,
											transformOrigin: "top left",
											backgroundImage: `url(${renderedHoverPreviewFrame.assetSignedUrl})`,
											backgroundPosition: `-${renderedHoverPreviewFrame.offsetXPx}px -${renderedHoverPreviewFrame.offsetYPx}px`,
											backgroundSize: `${renderedHoverPreviewFrame.sheetWidthPx}px ${renderedHoverPreviewFrame.sheetHeightPx}px`,
											backgroundRepeat: "no-repeat",
										}}
									/>
								</div>
							)}
							<div
								className={cn(
									"absolute bg-black/60 px-2 py-0.5 rounded text-sm left-1/2 -translate-x-1/2",
									renderedHoverPreviewFrame ? "-top-10" : "-top-8",
								)}
							>
								{formatPlayerTime(hoverState.time)}
							</div>
						</div>
					</>
				)}
			</div>
		</div>
	);
};
