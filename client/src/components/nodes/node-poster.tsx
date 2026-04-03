import { EllipsisVerticalIcon, FolderPlusIcon } from "lucide-react";
import type React from "react";
import { useState, type FC } from "react";
import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../../@generated/gql";
import type { NodePosterFragment } from "../../@generated/gql/graphql";
import { formatReleaseYear } from "../../lib/format-release-year";
import { getPathForNode } from "../../lib/getPathForMedia";
import { cn } from "../../lib/utils";
import { AddToCollectionModal } from "../add-to-collection-modal";
import { Image, ImageType } from "../image";
import { PlayWrapper } from "../play-wrapper";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "../ui/dropdown-menu";
import { UnplayedItemsTab } from "../unplayed-items-tab";

interface NodePosterProps {
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
			posterImage {
				...ImageAsset
			}
			firstAired
			lastAired
		}
		nextPlayable {
			id
			watchProgress {
				progressPercent
				completed
				updatedAt
			}
		}
		unplayedCount
		seasonCount
		episodeCount
		...GetPathForNode
	}
`);

export const NodePoster: FC<NodePosterProps> = ({ node: nodeRaw, className, style }) => {
	const node = unmask(Fragment, nodeRaw);
	const path = getPathForNode(node);
	const detail = getPosterDetail(node);
	const [isAddToCollectionOpen, setIsAddToCollectionOpen] = useState(false);

	return (
		<>
			<div className={cn("flex flex-col gap-2 overflow-hidden select-none", className)} style={style}>
				<PlayWrapper
					itemId={node.nextPlayable?.id ?? node.id}
					path={path}
					unavailable={node.unavailableAt != null}
					watchProgress={node.nextPlayable?.watchProgress ?? null}
				>
					<Image
						type={ImageType.Poster}
						asset={node.properties.posterImage}
						alt={node.properties.displayName}
						className="w-full"
					/>
					<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
				</PlayWrapper>
				<div className="flex items-start gap-2">
					<Link to={path} className="block min-w-0 flex-1 truncate text-sm group">
						<span className="group-hover:underline">{node.properties.displayName}</span>
						{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
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
							>
								<DropdownMenuItem className="py-2" onSelect={() => setIsAddToCollectionOpen(true)}>
									<FolderPlusIcon className="size-4" />
									Add to Collection
								</DropdownMenuItem>
							</DropdownMenuContent>
						</DropdownMenu>
					</div>
				</div>
			</div>
			<AddToCollectionModal
				nodeId={node.id}
				open={isAddToCollectionOpen}
				onOpenChange={setIsAddToCollectionOpen}
			/>
		</>
	);
};

const getPosterDetail = (node: NodePosterFragment): string | number | null => {
	if (node.kind === "SERIES") {
		if (node.seasonCount > 0) return `${node.seasonCount} ${node.seasonCount === 1 ? "season" : "seasons"}`;
		if (node.episodeCount > 0) return `${node.episodeCount} ${node.episodeCount === 1 ? "episode" : "episodes"}`;
	}

	if (!node.properties.firstAired && !node.properties.lastAired) return null;
	return formatReleaseYear(node.properties.firstAired, node.properties.lastAired ?? null) ?? null;
};
