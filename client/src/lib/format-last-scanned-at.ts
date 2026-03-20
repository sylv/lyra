import prettyMilliseconds from "pretty-ms";

export const formatLastScannedAt = (lastScannedAt: number | null) => {
	if (!lastScannedAt) {
		return "Never scanned";
	}

	const seenAt = new Date(lastScannedAt * 1000);
	const now = new Date();
	const elapsedMs = Math.max(0, now.getTime() - seenAt.getTime());
	return `Last scanned ${prettyMilliseconds(elapsedMs, { unitCount: 1, verbose: true })} ago`;
};
