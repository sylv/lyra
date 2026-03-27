export const formatReleaseYear = (releasedAt: number | null, endedAt: number | null) => {
	const start = releasedAt ? new Date(releasedAt * 1000).getFullYear() : null;
	const end = endedAt ? new Date(endedAt * 1000).getFullYear() : null;

	if (start && end) {
		return `${start} - ${end}`;
	}

	if (start) {
		return start;
	}
};
