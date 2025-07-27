import clsx from "clsx";
import { ImageIcon } from "lucide-react";
import type { FC } from "react";
import { getImageProxyUrl } from "../lib/getImageProxyUrl";

const BASE_CLASSES = "rounded-lg bg-zinc-700/30 shrink-0 aspect-[16/9] object-cover select-none";

interface ThumbnailProps {
	imageUrl: string | null | undefined;
	alt: string;
	className?: string;
}

export const Thumbnail: FC<ThumbnailProps> = ({ imageUrl, alt, className = "h-38" }) => {
	if (!imageUrl) {
		return (
			<div
				className={clsx(
					BASE_CLASSES,
					"flex flex-col justify-center items-center gap-2 text-zinc-500 p-4 overflow-hidden shrink-0",
					className,
				)}
			>
				<ImageIcon />
				<span className="text-sm text-center font-semibold">{alt}</span>
			</div>
		);
	}

	return (
		<img
			src={getImageProxyUrl(imageUrl, 600)}
			alt={alt}
			className={clsx(BASE_CLASSES, className)}
			loading="lazy"
			decoding="async"
		/>
	);
};
