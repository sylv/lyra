import { graphql, readFragment, type FragmentOf } from "gql.tada";
import type React from "react";
import type { FC } from "react";
import { getPathForRoot, GetPathForRootFrag } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { Image, ImageAssetFrag, ImageType } from "./image";
import { PlayWrapper } from "./play-wrapper";

interface MediaPosterProps {
	media: FragmentOf<typeof MediaPosterFrag>;
	className?: string;
	style?: React.CSSProperties;
}

export const MediaPosterFrag = graphql(
	`
	fragment MediaPoster on RootNode {
		id
		name
		kind
		properties {
			posterImage {
				...ImageAsset
			}
		}
		playableItem {
			id
		}
		watchProgress {
			progressPercent
			updatedAt
		}
		...GetPathForRoot
	}
`,
	[GetPathForRootFrag, ImageAssetFrag],
);

export const MediaPoster: FC<MediaPosterProps> = ({ media: mediaRaw, className, style }) => {
	const media = readFragment(MediaPosterFrag, mediaRaw);
	const path = getPathForRoot(media);

	return (
		<div className={cn("flex flex-col gap-2 overflow-hidden select-none", className)} style={style}>
			<PlayWrapper itemId={media.playableItem?.id} path={path} watchProgress={media.watchProgress}>
				<Image type={ImageType.Poster} asset={media.properties.posterImage} alt={media.name} className="w-full" />
			</PlayWrapper>
			<a
				href={path}
				className="block w-full truncate text-sm font-semibold text-zinc-400 transition-colors hover:text-zinc-300 hover:underline"
			>
				{media.name}
			</a>
		</div>
	);
};
