import { EllipsisVerticalIcon, FolderPlusIcon } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { getPathForNode } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { AddToCollectionModal } from "./add-to-collection-modal";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu";

export const CollectionNodeCardFragment = graphql(`
	fragment CollectionNodeCard on Node {
		id
		kind
		libraryId
		unavailableAt
		properties {
			displayName
			description
			posterImage {
				...ImageAsset
			}
			thumbnailImage {
				...ImageAsset
			}
			seasonNumber
			episodeNumber
			firstAired
			lastAired
		}
		watchProgress {
			progressPercent
			completed
			updatedAt
		}
		nextPlayable {
			id
			watchProgress {
				progressPercent
				completed
				updatedAt
			}
		}
		...GetPathForNode
	}
`);

export function CollectionNodeCard({
	node: nodeRaw,
	className,
}: {
	node: FragmentType<typeof CollectionNodeCardFragment>;
	className?: string;
}) {
	const node = unmask(CollectionNodeCardFragment, nodeRaw);
	const path = getPathForNode(node);
	const [isAddToCollectionOpen, setIsAddToCollectionOpen] = useState(false);

	if (node.kind === "EPISODE") {
		return (
			<>
				<div className={cn("flex w-44 shrink-0 flex-col gap-2", className)}>
					<PlayWrapper
						itemId={node.id}
						path={path}
						unavailable={node.unavailableAt != null}
						watchProgress={node.watchProgress}
					>
						<Image
							type={ImageType.Thumbnail}
							asset={node.properties.thumbnailImage ?? node.properties.posterImage}
							alt={node.properties.displayName}
							className="h-28 w-full rounded-sm object-cover"
						/>
					</PlayWrapper>
					<div className="flex items-start gap-2">
						<Link to={path} className="min-w-0 flex-1">
							<div className="line-clamp-2 text-sm font-semibold hover:underline">{node.properties.displayName}</div>
							<div className="text-xs text-zinc-500">
								S{node.properties.seasonNumber ?? "?"}E{node.properties.episodeNumber ?? "?"}
							</div>
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
				<AddToCollectionModal nodeId={node.id} open={isAddToCollectionOpen} onOpenChange={setIsAddToCollectionOpen} />
			</>
		);
	}

	return (
		<>
			<div className={cn("flex w-44 shrink-0 flex-col gap-2", className)}>
				<PlayWrapper
					itemId={node.nextPlayable?.id ?? node.id}
					path={path}
					unavailable={node.unavailableAt != null}
					watchProgress={node.nextPlayable?.watchProgress ?? null}
				>
					<Image
						type={ImageType.Poster}
						asset={node.properties.posterImage ?? node.properties.thumbnailImage}
						alt={node.properties.displayName}
						className="w-full rounded-sm object-cover"
					/>
				</PlayWrapper>
				<div className="flex items-start gap-2">
					<Link to={path} className="min-w-0 flex-1">
						<div className="line-clamp-2 text-sm font-semibold hover:underline">{node.properties.displayName}</div>
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
			<AddToCollectionModal nodeId={node.id} open={isAddToCollectionOpen} onOpenChange={setIsAddToCollectionOpen} />
		</>
	);
}
