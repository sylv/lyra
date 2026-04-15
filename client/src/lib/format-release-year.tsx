export const formatReleaseYear = (firstAired: number | null, lastAired: number | null) => {
  const start = firstAired ? new Date(firstAired * 1000).getFullYear() : null;
  const end = lastAired ? new Date(lastAired * 1000).getFullYear() : null;

  if (start && end && start !== end) {
    return `${start} - ${end}`;
  }

  return start ?? end ?? undefined;
};
