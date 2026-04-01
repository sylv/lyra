export interface TimelinePreviewAssetLike {
	id?: string | null;
	signedUrl: string;
	width: number | null;
	height: number | null;
}

export interface TimelinePreviewSheetLike {
	positionMs: number;
	endMs: number;
	sheetIntervalMs: number;
	sheetGapSize: number;
	asset: TimelinePreviewAssetLike;
}

export interface TimelinePreviewFrame {
	assetId: string | null;
	assetSignedUrl: string;
	sheetWidthPx: number;
	sheetHeightPx: number;
	frameWidthPx: number;
	frameHeightPx: number;
	offsetXPx: number;
	offsetYPx: number;
}

export const sortTimelinePreviewSheets = <TSheet extends TimelinePreviewSheetLike>(sheets: TSheet[]): TSheet[] => {
	return [...sheets].sort((a, b) => {
		if (a.positionMs !== b.positionMs) return a.positionMs - b.positionMs;
		return (a.asset.id ?? a.asset.signedUrl).localeCompare(b.asset.id ?? b.asset.signedUrl);
	});
};

export const getTimelinePreviewFrameAtMs = <TSheet extends TimelinePreviewSheetLike>(
	positionMs: number,
	sheets: TSheet[],
): TimelinePreviewFrame | null => {
	if (!Number.isFinite(positionMs) || positionMs < 0) return null;

	for (const sheet of sheets) {
		const sheetWidthPx = sheet.asset.width ?? 0;
		const sheetHeightPx = sheet.asset.height ?? 0;
		if (sheetWidthPx <= 0 || sheetHeightPx <= 0 || sheet.sheetGapSize < 0 || sheet.sheetIntervalMs <= 0) continue;

		const frameCount = Math.floor((sheet.endMs - sheet.positionMs) / sheet.sheetIntervalMs);
		if (frameCount <= 0) continue;

		// Timeline preview frames are offset by one interval, so the first frame represents +interval rather than 0ms.
		const previewTimestampMs = Math.max(
			sheet.sheetIntervalMs,
			Math.round(positionMs / sheet.sheetIntervalMs) * sheet.sheetIntervalMs,
		);
		if (previewTimestampMs <= sheet.positionMs || previewTimestampMs > sheet.endMs) continue;

		const columns = Math.max(1, Math.ceil(Math.sqrt(frameCount)));
		const rows = Math.max(1, Math.ceil(frameCount / columns));
		const frameWidthPx = Math.floor((sheetWidthPx - (columns + 1) * sheet.sheetGapSize) / columns);
		const frameHeightPx = Math.floor((sheetHeightPx - (rows + 1) * sheet.sheetGapSize) / rows);
		if (frameWidthPx <= 0 || frameHeightPx <= 0) continue;

		const rawIndex = Math.floor((previewTimestampMs - sheet.positionMs) / sheet.sheetIntervalMs) - 1;
		const frameIndex = Math.max(0, Math.min(frameCount - 1, rawIndex));
		const columnIndex = frameIndex % columns;
		const rowIndex = Math.floor(frameIndex / columns);

		return {
			assetId: sheet.asset.id ?? null,
			assetSignedUrl: sheet.asset.signedUrl,
			sheetWidthPx,
			sheetHeightPx,
			frameWidthPx,
			frameHeightPx,
			offsetXPx: sheet.sheetGapSize + columnIndex * (frameWidthPx + sheet.sheetGapSize),
			offsetYPx: sheet.sheetGapSize + rowIndex * (frameHeightPx + sheet.sheetGapSize),
		};
	}

	return null;
};
