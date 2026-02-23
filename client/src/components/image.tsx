import clsx from "clsx";
import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { ImageIcon } from "lucide-react";
import type { FC } from "react";
import { getThumbhashDataUrl } from "../lib/thumbhash";

export enum ImageType {
	Poster = "poster",
	Thumbnail = "thumbnail",
}

export const ImageAssetFrag = graphql(`
	fragment ImageAsset on Asset {
		id
		source
		sourceUrl
		hashSha256
		sizeBytes
		mimeType
		height
		width
		thumbhash
		createdAt
		deletedAt
	}
`);

export type ImageAsset = FragmentOf<typeof ImageAssetFrag>;

interface ImageTypeConfig {
	baseClasses: string;
	defaultClassName: string;
	fallbackTextClasses: string;
	proxyWidth: number;
}

const IMAGE_TYPE_CONFIG: Record<ImageType, ImageTypeConfig> = {
	[ImageType.Poster]: {
		baseClasses: "aspect-[2/3] bg-zinc-700/30 shrink-0 select-none",
		defaultClassName: "h-64",
		fallbackTextClasses:
			"max-w-[60%] text-sm text-center font-semibold whitespace-normal wrap-break-words wrap-anywhere",
		proxyWidth: 400,
	},
	[ImageType.Thumbnail]: {
		baseClasses: "bg-zinc-700/30 shrink-0 aspect-[16/9] object-cover select-none",
		defaultClassName: "h-38",
		fallbackTextClasses: "text-sm text-center font-semibold",
		proxyWidth: 600,
	},
};

interface ImageProps {
	type: ImageType;
	asset: ImageAsset | null | undefined;
	alt: string;
	className?: string;
}

export const getAssetImageUrl = (asset: ImageAsset | { id: number }, height: number): string => {
	const assetId = "id" in asset ? asset.id : readFragment(ImageAssetFrag, asset).id;
	const params = new URLSearchParams({
		height: String(height),
	});
	return `/api/assets/${assetId}?${params.toString()}`;
};

export const Image: FC<ImageProps> = ({ type, asset, alt, className }) => {
	const config = IMAGE_TYPE_CONFIG[type];
	const resolvedClassName = className ?? config.defaultClassName;
	const resolvedAsset = asset ? readFragment(ImageAssetFrag, asset) : null;
	const thumbhashPreview = getThumbhashDataUrl(resolvedAsset?.thumbhash);

	if (!resolvedAsset) {
		return (
			<div
				className={clsx(
					config.baseClasses,
					"flex flex-col justify-center items-center gap-2 text-zinc-500 p-4 overflow-hidden",
					resolvedClassName,
				)}
			>
				<ImageIcon />
				<span className={config.fallbackTextClasses}>{alt}</span>
			</div>
		);
	}

	return (
		<img
			src={getAssetImageUrl(resolvedAsset, config.proxyWidth)}
			alt={alt}
			className={clsx(config.baseClasses, resolvedClassName)}
			style={
				thumbhashPreview
					? {
							backgroundImage: `url(${thumbhashPreview})`,
							backgroundSize: "cover",
							backgroundPosition: "center",
							backgroundRepeat: "no-repeat",
						}
					: undefined
			}
		/>
	);
};
