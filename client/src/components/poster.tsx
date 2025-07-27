import clsx from "clsx";
import { ImageIcon } from "lucide-react";
import type { FC } from "react";
import { getImageProxyUrl } from "../lib/getImageProxyUrl";

const BASE_CLASSES = "rounded-lg aspect-[2/3] bg-zinc-700/30 shrink-0 select-none";

interface PosterProps {
	imageUrl: string | null | undefined;
	alt: string;
	className?: string;
}

export const Poster: FC<PosterProps> = ({ imageUrl, alt, className = "h-64" }) => {
	if (!imageUrl) {
		return (
			<div
				className={clsx(
					BASE_CLASSES,
					"flex flex-col justify-center items-center gap-2 text-zinc-500 p-4 overflow-hidden",
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
			src={getImageProxyUrl(imageUrl, 400)}
			alt={alt}
			className={clsx(BASE_CLASSES, className)}
			loading="lazy"
			decoding="async"
		/>
	);
};
