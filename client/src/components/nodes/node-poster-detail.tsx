import { EllipsisVerticalIcon } from "lucide-react";
import type React from "react";
import { type FC } from "react";
import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../../@generated/gql";
import { getPathForNode } from "../../lib/get-path-for-node";
import { cn } from "../../lib/utils";
import { Image, ImageType } from "../image";
import { PlayWrapper } from "../play-wrapper";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "../ui/dropdown-menu";
import { UnplayedItemsTab } from "../unplayed-items-tab";

interface NodePosterDetailProps {
  node: FragmentType<typeof Fragment>;
  className?: string;
  style?: React.CSSProperties;
}

const Fragment = graphql(`
  fragment NodePoster on Node {
    id
    kind
    libraryId
    unavailableAt
    properties {
      displayName
      displayDetail
      posterImage {
        ...ImageAsset
      }
      firstAired
      lastAired
    }
    currentPlayable {
      id
      watchProgressHint
    }
    unplayedCount
    seasonCount
    episodeCount
    seasonNumber
    episodeNumber
    ...GetPathForNode
  }
`);

export const NodePosterDetail: FC<NodePosterDetailProps> = ({ node: nodeRaw, className, style }) => {
  const node = unmask(Fragment, nodeRaw);
  const path = getPathForNode(node);
  const namePrefix = node.kind === "EPISODE" ? `S${node.seasonNumber}E${node.episodeNumber}` : "";

  return (
    <>
      <div className={cn("flex flex-col gap-2 overflow-hidden select-none", className)} style={style}>
        <PlayWrapper
          itemId={node.currentPlayable?.id ?? node.id}
          path={path}
          unavailable={node.unavailableAt != null}
          watchProgressHint={node.currentPlayable?.watchProgressHint ?? null}
        >
          <Image type={ImageType.Poster} asset={node.properties.posterImage} alt={node.properties.displayName} />
          <UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
        </PlayWrapper>
        <div className="flex items-start gap-2">
          <Link to={path} className="block min-w-0 flex-1 truncate text-sm group">
            <span className="group-hover:underline">
              {namePrefix} {node.properties.displayName}
            </span>
            {node.properties.displayDetail && (
              <p className="text-xs text-zinc-500 -mt-0.5">{node.properties.displayDetail}</p>
            )}
          </Link>
          <div className="shrink-0">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <button
                  type="button"
                  className="rounded-sm p-1 text-zinc-400 transition hover:bg-zinc-500/20 hover:text-zinc-100"
                  aria-label={`Actions for ${node.properties.displayName}`}
                >
                  <EllipsisVerticalIcon className="size-4" />
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align="end"
                className="border-zinc-800 bg-black/95 text-zinc-100 shadow-xl shadow-black/40"
              ></DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>
    </>
  );
};
