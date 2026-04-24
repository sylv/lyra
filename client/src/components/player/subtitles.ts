import type { CaptionsFileFormat } from "media-captions";
import { SubtitleRenditionType, type ItemPlaybackQuery } from "../../@generated/gql/graphql";

export type PlaybackSubtitleTrack = NonNullable<
  NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["defaultFile"]>["playbackOptions"]>["subtitleTracks"][number]
>;

export type PlaybackSubtitleRendition = PlaybackSubtitleTrack["renditions"][number];

const subtitleFormatRank = (format: CaptionsFileFormat) => {
  switch (format) {
    case "vtt":
      return 0;
    case "srt":
      return 1;
    case "ass":
    case "ssa":
      return 2;
  }
};

export const subtitleRenditionFormat = (rendition: Pick<PlaybackSubtitleRendition, "codecName">): CaptionsFileFormat | null => {
  switch (rendition.codecName.trim().toLowerCase()) {
    case "webvtt":
    case "vtt":
      return "vtt";
    case "srt":
    case "subrip":
      return "srt";
    case "ass":
      return "ass";
    case "ssa":
      return "ssa";
    default:
      return null;
  }
};

const subtitleRenditionRank = (rendition: PlaybackSubtitleRendition) => {
  const format = subtitleRenditionFormat(rendition);
  if (!format) return Number.POSITIVE_INFINITY;

  const formatRank = subtitleFormatRank(format);
  if (rendition.type === SubtitleRenditionType.Direct) {
    return formatRank;
  }
  if (rendition.type === SubtitleRenditionType.Generated) {
    return 20 + formatRank;
  }
  return 10 + formatRank;
};

export const listPreferredSubtitleRenditions = (track: PlaybackSubtitleTrack) =>
  [...track.renditions]
    .filter((rendition) => Number.isFinite(subtitleRenditionRank(rendition)))
    .sort((left, right) => subtitleRenditionRank(left) - subtitleRenditionRank(right));

export const pickPreferredSubtitleRendition = (track: PlaybackSubtitleTrack) =>
  listPreferredSubtitleRenditions(track)[0] ?? null;
