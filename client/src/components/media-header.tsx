import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type { FC } from "react";
import { useDynamicBackground } from "../hooks/use-background";
import { formatReleaseYear } from "../lib/format-release-year";
import { getImageProxyUrl } from "../lib/getImageProxyUrl";
import { getPathForRoot, GetPathForRootFrag } from "../lib/getPathForMedia";
import { PlayWrapper } from "./play-wrapper";
import { Poster } from "./poster";
import { Skeleton } from "./skeleton";

interface MediaHeaderProps {
	media: FragmentOf<typeof MediaHeaderFrag>;
}

export const MediaHeaderFrag = graphql(
	`
	fragment MediaHeader on RootNode {
		id
		name
		properties {
			posterUrl
			backgroundUrl
			releasedAt
			endedAt
			runtimeMinutes
			description
		}
		playable_item {
			id
		}
		watchProgress {
			progressPercent
			updatedAt
		}
		...GetPathForRoot
	}
`,
	[GetPathForRootFrag],
);

export const MediaHeader: FC<MediaHeaderProps> = ({ media: mediaRaw }) => {
	const media = readFragment(MediaHeaderFrag, mediaRaw);
	const dynamicUrl = media.properties.backgroundUrl
		? getImageProxyUrl(media.properties.backgroundUrl, 200)
		: null;
	const path = getPathForRoot(media);

	useDynamicBackground(dynamicUrl);

	return (
		<div className="bg-zinc-800/30 border-700/30 p-6 border-b">
			<div className="flex gap-6 container mx-auto">
				<PlayWrapper itemId={media.playable_item?.id} path={path} watchProgress={media.watchProgress}>
					<Poster imageUrl={media.properties.posterUrl} alt={media.name} className="h-72" />
				</PlayWrapper>
				<div className="flex flex-col gap-2 justify-between">
					<div className="flex flex-col gap-2">
						<h1 className="text-2xl font-bold">
							{media.name}
							{media.properties.releasedAt && (
								<span className="text-zinc-400 ml-2 text-lg">
									{formatReleaseYear(media.properties.releasedAt, media.properties.endedAt ?? null)}
								</span>
							)}
						</h1>
						{media.properties.runtimeMinutes && (
							<p className="text-sm text-zinc-400">{media.properties.runtimeMinutes} minutes</p>
						)}
						<p className="text-sm text-zinc-400">{media.properties.description || "No description for this"}</p>
					</div>
				</div>
			</div>
		</div>
	);
};

export const MediaHeaderSkeleton: FC = () => {
	return (
		<div className="bg-zinc-800/30 border-700/30 p-6 border-b">
			<div className="flex gap-6 container mx-auto">
				<Skeleton className="h-72 aspect-[2/3] rounded-md" />
				<div className="flex flex-col gap-2 justify-between">
					<div className="flex flex-col gap-2">
						<div className="flex items-center gap-2">
							<Skeleton className="h-8 w-64" />
							<Skeleton className="h-6 w-20" />
						</div>
						<Skeleton className="h-4 w-32" />
						<div className="space-y-1">
							<Skeleton className="h-4 w-full" />
							<Skeleton className="h-4 w-full" />
							<Skeleton className="h-4 w-3/4" />
						</div>
					</div>
					<div className="flex gap-2">
						<Skeleton className="h-10 w-32 rounded-lg" />
					</div>
				</div>
			</div>
		</div>
	);
};
