import { AddToCollectionModal } from "@/components/add-to-collection-modal";
import { Button, ButtonSize, ButtonStyle } from "@/components/button";
import { FilterButton } from "@/components/filter-button";
import { Image, ImageType } from "@/components/image";
import { PlayWrapper } from "@/components/play-wrapper";
import { UnplayedItemsTab } from "@/components/unplayed-items-tab";
import { WatchlistButton } from "@/components/watchlist-controls";
import { useDynamicBackground } from "@/hooks/use-background";
import { ChevronRightIcon, FolderPlusIcon, PlayIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Link, Navigate, useParams } from "react-router";
import { graphql } from "../@generated/gql";
import { NodeAvailability, NodeKind, OrderBy } from "../@generated/gql/graphql";
import { DisplayKind, NodeList } from "../components/nodes/node-list";
import { NodePosterDetail } from "../components/nodes/node-poster-detail";
import { openPlayerMedia } from "../components/player/player-context";
import { ShelfCarousel } from "../components/shelf-carousel";
import { useSuspenseQuery } from "../hooks/use-suspense-query";
import { useTitle } from "../hooks/use-title";
import { getPathForNode } from "../lib/getPathForMedia";

const Query = graphql(`
  query GetNodeById($nodeId: String!) {
    node(nodeId: $nodeId) {
      id
      kind
      inWatchlist
      unavailableAt
      unplayedCount
      seasonCount
      seasonNumber
      episodeNumber
      ...GetPathForNode
      parent {
        id
        ...GetPathForNode
      }
      root {
        id
        ...GetPathForNode
        properties {
          displayName
        }
      }
      properties {
        displayName
        tagline
        posterImage {
          ...ImageAsset
        }
        thumbnailImage {
          ...ImageAsset
        }
        logoImage {
          id
          signedUrl
          aspectRatio
        }
        backdropImage {
          ...ImageAsset
          signedUrl
          aspectRatio
        }
        description
        contentRating {
          rating
        }
        genres {
          name
        }
        cast {
          characterName
          department
          person {
            id
            name
            profileImage {
              ...ImageAsset
            }
          }
        }
      }
      watchProgress {
        id
        progressPercent
        completed
        updatedAt
      }
      nextPlayable {
        id
        seasonNumber
        episodeNumber
        watchProgress {
          id
          progressPercent
          completed
          updatedAt
        }
      }
      defaultFile {
        probe {
          runtimeMinutes
          width
          height
          videoCodec
          videoBitrate
        }
        subtitles {
          displayName
          languageBcp47
        }
      }
      recommendedNodes {
        id
        ...NodePoster
      }
    }
  }
`);

function formatResolution(width?: number | null, height?: number | null): string | null {
  if (!height) return null;
  if (height >= 2160) return "4K";
  if (height >= 1440) return "1440p";
  if (height >= 1080) return "1080p";
  if (height >= 720) return "720p";
  if (height >= 480) return "480p";
  if (width) return `${width}×${height}`;
  return `${height}p`;
}

function formatVideoCodec(codec?: string | null): string | null {
  if (!codec) return null;
  switch (codec.toLowerCase()) {
    case "h264":
      return "H.264";
    case "h265":
      return "H.265";
    case "av1":
      return "AV1";
    default:
      return codec.toUpperCase();
  }
}

function formatBitrate(bitsPerSecond?: number | null): string | null {
  if (!bitsPerSecond) return null;
  return `${(bitsPerSecond / 1_000_000).toFixed(1)} Mbps`;
}

type SubtitleEntry = { displayName: string; languageBcp47?: string | null };

function formatSubtitles(tracks: SubtitleEntry[]): string | null {
  if (tracks.length === 0) return null;
  // Deduplicate by label, prefer shorter unique label list
  const seen = new Set<string>();
  const labels: string[] = [];
  for (const t of tracks) {
    const key = t.displayName;
    if (!seen.has(key)) {
      seen.add(key);
      labels.push(t.displayName);
    }
  }
  return labels.join(", ");
}

type NodeDetailsSectionProps = {
  node: {
    defaultFile?: {
      probe?: {
        width?: number | null;
        height?: number | null;
        videoCodec?: string | null;
        videoBitrate?: number | null;
      } | null;
      subtitles: SubtitleEntry[];
    } | null;
  };
};

const NodeDetailsSection = ({ node }: NodeDetailsSectionProps) => {
  const probe = node.defaultFile?.probe;
  const resolution = formatResolution(probe?.width, probe?.height);
  const codec = formatVideoCodec(probe?.videoCodec);
  const bitrate = formatBitrate(probe?.videoBitrate);
  const subtitleStr = node.defaultFile ? formatSubtitles(node.defaultFile.subtitles) : null;

  if (!resolution && !codec && !bitrate && !subtitleStr) return null;

  return (
    <div className="container">
      <span className="text-xl font-semibold">Details</span>
      <dl className="mt-3 flex flex-col gap-1.5 text-xs">
        {resolution && (
          <div className="flex gap-6">
            <dt className="text-zinc-400 w-32 shrink-0">Video Resolution</dt>
            <dd className="text-zinc-100">{resolution}</dd>
          </div>
        )}
        {codec && (
          <div className="flex gap-6">
            <dt className="text-zinc-400 w-32 shrink-0">Video Codec</dt>
            <dd className="text-zinc-100">{codec}</dd>
          </div>
        )}
        {bitrate && (
          <div className="flex gap-6">
            <dt className="text-zinc-400 w-32 shrink-0">Video Bitrate</dt>
            <dd className="text-zinc-100">{bitrate}</dd>
          </div>
        )}
        {subtitleStr && (
          <div className="flex gap-6">
            <dt className="text-zinc-400 w-32 shrink-0">Subtitles</dt>
            <dd className="text-zinc-100 max-w-[calc(max(20vw,200px))]">{subtitleStr}</dd>
          </div>
        )}
      </dl>
    </div>
  );
};

const BackdropOverlay = ({
  node,
}: {
  node: { properties: { backdropImage?: { signedUrl: string } | null; displayName: string } };
}) => {
  if (!node.properties.backdropImage) return null;
  return (
    <div
      className="absolute -z-10 -bottom-12 -right-23 -top-6 left-0 opacity-15"
      style={{
        WebkitMaskImage: `
					linear-gradient(to right,
						transparent 0%,
						black 30%,
						black 100%
					),
					linear-gradient(to bottom,
						black 0%,
						black 30%,
						transparent 100%
					)
				`,
        maskImage: `
					linear-gradient(to right,
						transparent 0%,
						black 30%,
						black 100%
					),
					linear-gradient(to bottom,
						black 0%,
						black 30%,
						transparent 100%
					)
				`,
        WebkitMaskComposite: "source-in",
        maskComposite: "intersect",
      }}
    >
      <img
        src={node.properties.backdropImage.signedUrl}
        alt={`${node.properties.displayName} backdrop`}
        className="h-full w-full object-cover"
      />
    </div>
  );
};

export function LibraryNodeRoute() {
  const { nodeId } = useParams<{ nodeId: string }>();
  const queryVariables = useMemo(() => ({ nodeId: nodeId! }), [nodeId]);
  const [{ data }] = useSuspenseQuery({
    query: Query,
    variables: queryVariables,
  });

  const [isAddToCollectionOpen, setIsAddToCollectionOpen] = useState(false);
  const [selectedSeasonNumbers, setSelectedSeasonNumbers] = useState<number[]>([1]);
  const node = data?.node;

  useTitle(node?.root?.properties.displayName ?? node?.properties.displayName);
  useDynamicBackground(
    (node?.kind === NodeKind.Episode ? node.properties.thumbnailImage : node?.properties.posterImage) || null,
  );

  const playText = useMemo(() => {
    if (!node?.nextPlayable) return "Play";

    const parts = [];
    if (node.nextPlayable.seasonNumber && node.nextPlayable.episodeNumber) {
      parts.push(`S${node.nextPlayable.seasonNumber}E${node.nextPlayable.episodeNumber}`);
    }

    if (node.nextPlayable.watchProgress?.progressPercent) parts.unshift("Resume");
    else parts.unshift("Play");

    return parts.join(" ");
  }, [node?.nextPlayable]);

  if (!node) return null;

  const allSeasonNumbers = Array.from({ length: node.seasonCount }, (_, i) => i + 1);
  const showAllSeasonsButton = node.seasonCount > 1;
  const allSeasonsSelected =
    selectedSeasonNumbers.length === allSeasonNumbers.length &&
    allSeasonNumbers.every((seasonNumber) => selectedSeasonNumbers.includes(seasonNumber));

  const toggleSeason = (seasonNumber: number, additive: boolean) => {
    if (!additive) {
      setSelectedSeasonNumbers([seasonNumber]);
      return;
    }

    setSelectedSeasonNumbers((prev) => {
      if (prev.includes(seasonNumber)) return prev.filter((value) => value !== seasonNumber);
      return [...prev, seasonNumber].sort((a, b) => a - b);
    });
  };

  if (node.kind === NodeKind.Season && node.parent) {
    const path = getPathForNode(node.parent);
    return <Navigate to={path} replace={true} />;
  }

  const nodePath = getPathForNode(node);

  if (node.kind === NodeKind.Episode) {
    const rootPath = node.root ? getPathForNode(node.root) : null;
    const episodePlayText = node.watchProgress?.progressPercent ? "Resume" : "Play";
    const runtimeMinutes = node.defaultFile?.probe?.runtimeMinutes;

    return (
      <>
        <div className="pt-6 space-y-6 pb-36">
          <div className="container flex flex-col items-start lg:flex-row lg:gap-8 relative">
            <BackdropOverlay node={node} />
            <div className="shrink-0 hidden lg:block">
              <PlayWrapper
                itemId={node.id}
                path={nodePath}
                unavailable={node.unavailableAt != null}
                watchProgress={node.watchProgress}
              >
                <Image
                  type={ImageType.Thumbnail}
                  asset={node.properties.thumbnailImage}
                  alt={node.properties.displayName}
                  className="w-96"
                />
              </PlayWrapper>
            </div>
            <div className="flex w-full flex-col gap-2 relative">
              <div className="mb-8 flex flex-col gap-2">
                {rootPath && node.root && (
                  <div className="flex items-center gap-1.5 text-sm text-zinc-400">
                    <Link to={rootPath} className="hover:text-zinc-100 transition-colors">
                      {node.root.properties.displayName}
                    </Link>
                    <ChevronRightIcon className="size-3 shrink-0" />
                    <span>
                      S{node.seasonNumber}E{node.episodeNumber}
                    </span>
                  </div>
                )}
                <h1 className="text-2xl font-bold">{node.properties.displayName}</h1>
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    style={ButtonStyle.Primary}
                    size={ButtonSize.Smol}
                    className="w-fit"
                    icon={["play", PlayIcon]}
                    iconSide="left"
                    onClick={() => openPlayerMedia(node.id, true)}
                  >
                    {episodePlayText}
                  </Button>
                  <Button
                    style={ButtonStyle.Glass}
                    size={ButtonSize.Smol}
                    className="w-fit"
                    icon={["add-to-collection", FolderPlusIcon]}
                    iconSide="left"
                    onClick={() => setIsAddToCollectionOpen(true)}
                  >
                    Add to Collection
                  </Button>
                  <WatchlistButton nodeId={node.id} inWatchlist={node.inWatchlist} />
                </div>
                <div className="flex items-center gap-3">
                  {runtimeMinutes && <p className="text-sm text-zinc-400">{runtimeMinutes} minutes</p>}
                  {node.properties.contentRating && (
                    <p className="text-sm text-zinc-400">{node.properties.contentRating.rating}</p>
                  )}
                  {node.properties.genres.map((genre) => (
                    <p key={genre.name} className="text-sm text-zinc-400">
                      {genre.name}
                    </p>
                  ))}
                </div>
                {node.properties.description && (
                  <p className="text-sm text-zinc-400 lg:max-w-[35vw]">{node.properties.description}</p>
                )}
              </div>
            </div>
          </div>
          {node.properties.cast.length > 0 && (
            <div className="container">
              <ShelfCarousel title={<span className="text-xl font-semibold">Cast</span>}>
                {/* todo: should be clickable */}
                {node.properties.cast.map((castEntry, index) => (
                  <div key={index} className="min-w-0 flex-[0_0_8.25rem]">
                    <Image
                      type={ImageType.Avatar}
                      asset={castEntry.person.profileImage}
                      alt={castEntry.person.name}
                      className="w-full"
                    />
                    <div className="mt-2 text-sm">{castEntry.person.name}</div>
                    <div className="text-xs text-zinc-400">
                      {castEntry.characterName ? `as ${castEntry.characterName}` : castEntry.department}
                    </div>
                  </div>
                ))}
              </ShelfCarousel>
            </div>
          )}
          <NodeDetailsSection node={node} />
        </div>
        <AddToCollectionModal nodeId={node.id} open={isAddToCollectionOpen} onOpenChange={setIsAddToCollectionOpen} />
      </>
    );
  }

  const playableItemId = node.nextPlayable?.id;
  const playableWatchProgress =
    node.nextPlayable?.watchProgress ?? (playableItemId === node.id ? node.watchProgress : null);
  const runtimeMinutes = node.defaultFile?.probe?.runtimeMinutes;

  return (
    <>
      <div className="pt-6 space-y-6 pb-36">
        <div className="container flex flex-col items-end lg:flex-row lg:gap-8 relative">
          <BackdropOverlay node={node} />
          <div className="shrink-0 hidden lg:block">
            <PlayWrapper
              itemId={playableItemId}
              path={nodePath}
              unavailable={node.unavailableAt != null}
              watchProgress={playableWatchProgress}
            >
              <Image
                type={ImageType.Poster}
                asset={node.properties.posterImage}
                alt={node.properties.displayName}
                className="h-92"
              />
              <UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
            </PlayWrapper>
          </div>
          <div className="flex w-full flex-col gap-2 relative">
            <div className="my-8 flex flex-col gap-2">
              {!node.properties.logoImage && <h1 className="text-2xl font-bold">{node.properties.displayName}</h1>}
              {node.properties.logoImage && (
                <img
                  className="h-20 w-fit mb-4"
                  src={node.properties.logoImage.signedUrl}
                  alt={node.properties.displayName}
                  style={{
                    aspectRatio: node.properties.logoImage.aspectRatio || undefined,
                  }}
                />
              )}
              <div className="flex flex-wrap items-center gap-2">
                {node.nextPlayable && (
                  <Button
                    style={ButtonStyle.Primary}
                    size={ButtonSize.Smol}
                    className="w-fit"
                    icon={["play", PlayIcon]}
                    iconSide="left"
                    onClick={() => openPlayerMedia(node.nextPlayable!.id, true)}
                  >
                    {playText}
                  </Button>
                )}
                <Button
                  style={ButtonStyle.Glass}
                  size={ButtonSize.Smol}
                  className="w-fit"
                  icon={["add-to-collection", FolderPlusIcon]}
                  iconSide="left"
                  onClick={() => setIsAddToCollectionOpen(true)}
                >
                  Add to Collection
                </Button>
                <WatchlistButton nodeId={node.id} inWatchlist={node.inWatchlist} />
              </div>
              <div className="flex items-center gap-3">
                {runtimeMinutes && <p className="text-sm text-zinc-400">{runtimeMinutes} minutes</p>}
                {/* todo: should be clickable */}
                {node.properties.genres.map((genre) => (
                  <p key={genre.name} className="text-sm text-zinc-400">
                    {genre.name}
                  </p>
                ))}
              </div>
              {node.properties.tagline && <p className="text-sm italic text-zinc-300">{node.properties.tagline}</p>}
              <p className="text-sm text-zinc-400 lg:max-w-[35vw]">{node.properties.description}</p>
            </div>
          </div>
        </div>
        {node.kind === NodeKind.Series && (
          <div className="container">
            <span className="text-xl font-semibold">Episodes</span>
            <div className="mt-3 flex flex-wrap gap-2">
              {showAllSeasonsButton && (
                <FilterButton active={allSeasonsSelected} onClick={() => setSelectedSeasonNumbers(allSeasonNumbers)}>
                  All Seasons
                </FilterButton>
              )}
              {allSeasonNumbers.map((seasonNumber) => (
                <FilterButton
                  key={seasonNumber}
                  active={selectedSeasonNumbers.includes(seasonNumber)}
                  onClick={(event) => toggleSeason(seasonNumber, event.shiftKey)}
                >
                  Season {seasonNumber}
                </FilterButton>
              ))}
            </div>
            <div className="mt-2 flex flex-wrap gap-4">
              <div className="relative w-full">
                <div
                  className="grid gap-4"
                  style={{ gridTemplateColumns: "repeat(auto-fill, minmax(clamp(200px, 50vw, 260px), 1fr))" }}
                >
                  <NodeList
                    displayKind={DisplayKind.Episode}
                    filter={{
                      rootId: node.id,
                      kinds: [NodeKind.Episode],
                      orderBy: OrderBy.Order,
                      availability: NodeAvailability.Available,
                      seasonNumbers: selectedSeasonNumbers,
                    }}
                  />
                </div>
              </div>
            </div>
          </div>
        )}
        {node.properties.cast.length > 0 && (
          <div className="container">
            <ShelfCarousel title={<span className="text-xl font-semibold">Cast</span>}>
              {/* todo: should be clickable */}
              {node.properties.cast.map((castEntry, index) => (
                <div key={index} className="min-w-0 flex-[0_0_8.25rem]">
                  <Image
                    type={ImageType.Avatar}
                    asset={castEntry.person.profileImage}
                    alt={castEntry.person.name}
                    className="w-full"
                  />
                  <div className="mt-2 text-sm">{castEntry.person.name}</div>
                  <div className="text-xs text-zinc-400">
                    {castEntry.characterName ? `as ${castEntry.characterName}` : castEntry.department}
                  </div>
                </div>
              ))}
            </ShelfCarousel>
          </div>
        )}
        <NodeDetailsSection node={node} />
        {node.recommendedNodes.length > 0 && (
          <div className="container">
            <ShelfCarousel title={<span className="text-xl font-semibold">You might also like</span>}>
              {node.recommendedNodes.map((rec) => (
                <NodePosterDetail key={rec.id} node={rec} className="min-w-0 flex-[0_0_8.25rem]" />
              ))}
            </ShelfCarousel>
          </div>
        )}
      </div>
      <AddToCollectionModal nodeId={node.id} open={isAddToCollectionOpen} onOpenChange={setIsAddToCollectionOpen} />
    </>
  );
}
