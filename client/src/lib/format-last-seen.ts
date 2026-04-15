import prettyMilliseconds from "pretty-ms";

export const formatLastSeen = (lastSeenAt?: number | null) => {
  if (!lastSeenAt) {
    return "Not signed in yet";
  }

  const seenAt = new Date(lastSeenAt * 1000);
  const now = new Date();

  if (
    seenAt.getFullYear() === now.getFullYear() &&
    seenAt.getMonth() === now.getMonth() &&
    seenAt.getDate() === now.getDate()
  ) {
    return "Last seen today";
  }

  const elapsedMs = Math.max(0, now.getTime() - seenAt.getTime());
  return `Last seen ${prettyMilliseconds(elapsedMs, { unitCount: 1, verbose: true })} ago`;
};
