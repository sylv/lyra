import { useMemo, useState, type FC } from "react";
import { useNavigate } from "react-router";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { getPathForNode } from "../lib/getPathForMedia";
import { AddToCollectionModal } from "./add-to-collection-modal";
import { Image, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";
import { openPlayerMedia } from "./player/player-context";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { EllipsisVerticalIcon, FolderPlusIcon } from "lucide-react";

interface EpisodeCardProps {
	episode: FragmentType<typeof Fragment>;
}

const formatRuntime = (minutes: number | null) => {
	if (!minutes) return null;
	const hours = Math.floor(minutes / 60);
	const mins = minutes % 60;
	return hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
};

const Fragment = graphql(`
	fragment EpisodeCard on Node {
		id
		unavailableAt
		properties {
			displayName
			description
			thumbnailImage {
				...ImageAsset
			}
			seasonNumber
			episodeNumber
			firstAired
			runtimeMinutes
		}
		watchProgress {
			id
			progressPercent
			completed
			updatedAt
		}
		...GetPathForNode
	}
`);

export const EpisodeCard: FC<EpisodeCardProps> = ({ episode: episodeRef }) => {
	const episode = unmask(Fragment, episodeRef);
	const navigate = useNavigate();
	const path = getPathForNode(episode);
	const [isAddToCollectionOpen, setIsAddToCollectionOpen] = useState(false);
	const unavailable = episode.unavailableAt != null;
	const releaseDate = useMemo(() => {
		if (!episode.properties.firstAired) return null;
		return new Date(episode.properties.firstAired * 1000).toLocaleDateString(undefined, {
			year: "numeric",
			month: "short",
			day: "numeric",
		});
	}, [episode.properties.firstAired]);

	return (
		<>
			<div className="flex w-full gap-3">
				<button
					type="button"
					className="group/play flex min-w-0 flex-1 gap-4 text-left"
					aria-label={`Play ${episode.properties.displayName}`}
					disabled={unavailable}
					onClick={() => {
						if (unavailable) return;
						openPlayerMedia(episode.id, true);
						navigate(path);
					}}
				>
					<div className="relative h-min shrink-0 overflow-hidden rounded-sm">
						<PlayWrapper
							itemId={episode.id}
							path={path}
							unavailable={episode.unavailableAt != null}
							watchProgress={episode.watchProgress}
						>
							<Image
								type={ImageType.Thumbnail}
								asset={episode.properties.thumbnailImage}
								alt={episode.properties.displayName}
								className="aspect-[12:8] h-20 md:h-36"
							/>
						</PlayWrapper>
					</div>
					<div className="min-w-0">
						<h3 className="flex gap-3 font-semibold text-white">
							<div className="text-zinc-300">
								S{episode.properties.seasonNumber}E{episode.properties.episodeNumber}
							</div>
							{episode.properties.displayName}
						</h3>
						<div className="mb-2 flex items-center gap-3 text-sm text-zinc-400">
							{releaseDate && <div>{releaseDate}</div>}
							{episode.properties.runtimeMinutes && <div>{formatRuntime(episode.properties.runtimeMinutes)}</div>}
						</div>
						<p className="line-clamp-3 text-xs text-zinc-300">
							{episode.properties.description || "No description available"}
						</p>
					</div>
				</button>
				<div className="shrink-0">
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<button
								type="button"
								className="rounded-sm p-1 text-zinc-400 transition hover:bg-zinc-500/20 hover:text-zinc-100"
								aria-label={`Actions for ${episode.properties.displayName}`}
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
			<AddToCollectionModal nodeId={episode.id} open={isAddToCollectionOpen} onOpenChange={setIsAddToCollectionOpen} />
		</>
	);
};
