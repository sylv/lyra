import clsx from "clsx";
import { ImageIcon } from "lucide-react";
import type { FC } from "react";
import { getImageProxyUrl } from "../lib/getImageProxyUrl";

export enum ImageType {
	Poster = "poster",
	Thumbnail = "thumbnail",
}

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
		fallbackTextClasses: "max-w-[60%] text-sm text-center font-semibold whitespace-normal wrap-break-words wrap-anywhere",
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
	imageUrl: string | null | undefined;
	alt: string;
	className?: string;
}

export const Image: FC<ImageProps> = ({ type, imageUrl, alt, className }) => {
	const config = IMAGE_TYPE_CONFIG[type];
	const resolvedClassName = className ?? config.defaultClassName;

	if (!imageUrl) {
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
			src={getImageProxyUrl(imageUrl, config.proxyWidth)}
			alt={alt}
			className={clsx(config.baseClasses, resolvedClassName)}
			loading="lazy"
			decoding="async"
		/>
	);
};
