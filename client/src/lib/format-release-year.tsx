export const formatReleaseYear = (startYear: number | null, endYear: number | null) => {
	const start = startYear ? new Date(startYear * 1000).getFullYear() : null;
	const end = endYear ? new Date(endYear * 1000).getFullYear() : null;

	if (start && end) {
		return `${start} - ${end}`;
	}

	if (start) {
		return start;
	}
};
