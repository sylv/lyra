import type { FC } from "react";
import { useNavigate } from "react-router";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import { formatReleaseYear } from "../../../lib/format-release-year";
import { getPathForNode } from "../../../lib/getPathForMedia";
import { cn } from "../../../lib/utils";
import { togglePlayerFullscreen, usePlayerRuntimeStore } from "../player-runtime-store";

export const PlayerMetadataFragment = graphql(`
  fragment PlayerMetadata on Node {
    id
    libraryId
    root {
      id
      libraryId
      properties {
        displayName
      }
    }
    properties {
      displayName
      seasonNumber
      episodeNumber
      firstAired
      lastAired
    }
    ...GetPathForNode
  }
`);

export const PlayerMetadata: FC<{ media: FragmentType<typeof PlayerMetadataFragment> }> = ({ media: mediaRaw }) => {
  const media = unmask(PlayerMetadataFragment, mediaRaw);
  const navigate = useNavigate();
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);

  const detailsPath = media.libraryId ? getPathForNode(media) : null;
  const hasEpisodeMetadata =
    !!media.root?.properties.displayName &&
    media.properties.seasonNumber != null &&
    media.properties.episodeNumber != null;

  const title = hasEpisodeMetadata ? media.root?.properties.displayName ?? media.properties.displayName : media.properties.displayName;
  const description = hasEpisodeMetadata
    ? `S${media.properties.seasonNumber}E${media.properties.episodeNumber} ${media.properties.displayName}`
    : formatReleaseYear(media.properties.firstAired, media.properties.lastAired);

  return (
    <button
      type="button"
      className={cn("group rounded-sm px-3 py-2 text-left transition-colors", detailsPath ? "cursor-pointer" : "cursor-default")}
      onClick={(event) => {
        event.stopPropagation();
        if (!detailsPath) return;
        togglePlayerFullscreen(false);
        navigate(detailsPath);
      }}
    >
      <h2 className={cn("font-semibold text-white", detailsPath && "group-hover:underline", isFullscreen ? "text-xl" : "text-sm")}>
        {title}
      </h2>
      <p className={cn("text-gray-300", isFullscreen ? "text-sm" : "text-xs")}>{description}</p>
    </button>
  );
};
