export function parsePlaylistSegments(playlistText) {
  const lines = playlistText.split(/\r?\n/).map((line) => line.trim());
  const segments = [];
  let currentStart = 0;
  let pendingDuration = null;

  for (const line of lines) {
    if (!line) continue;
    if (line.startsWith("#EXTINF:")) {
      const durationStr = line.slice("#EXTINF:".length).split(",")[0];
      const duration = Number.parseFloat(durationStr);
      if (!Number.isFinite(duration)) {
        throw new Error(`Invalid EXTINF duration: ${durationStr}`);
      }
      pendingDuration = duration;
      continue;
    }

    if (pendingDuration != null && !line.startsWith("#")) {
      const match = line.match(/segment_(\d+)\.m4s/);
      const id = match ? Number.parseInt(match[1], 10) : segments.length;
      segments.push({
        id,
        start: currentStart,
        duration: pendingDuration,
      });
      currentStart += pendingDuration;
      pendingDuration = null;
    }
  }

  return segments;
}
