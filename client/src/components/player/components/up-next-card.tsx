// oxlint-disable jsx_a11y/click-events-have-key-events
// oxlint-disable jsx_a11y/click-events-have-key-events
import { type FC } from "react";
import type { FragmentType } from "../../../@generated/gql";
import { cn } from "../../../lib/utils";
import { Image, ImageType, Fragment as ImageAssetFragment } from "../../image";

interface UpNextCardProps {
	displayName: string;
	description: string | null | undefined;
	thumbnailImage: FragmentType<typeof ImageAssetFragment> | null | undefined;
	seasonNumber: number | null | undefined;
	episodeNumber: number | null | undefined;
	// when provided, shows the play button (and optionally Cancel)
	onPlay?: () => void;
	onCancel?: () => void;
	// 0–1, fills the play button background
	progressPercent?: number;
	// when > 0, button reads "Playing in Xs" instead of "Play now"
	countdownSeconds?: number;
}

export const UpNextCard: FC<UpNextCardProps> = ({
	displayName,
	description,
	thumbnailImage,
	seasonNumber,
	episodeNumber,
	onPlay,
	onCancel,
	progressPercent,
	countdownSeconds,
}) => {
	const titleParts: string[] = [];
	if (typeof seasonNumber === "number" && typeof episodeNumber === "number") {
		titleParts.push(`S${seasonNumber}E${episodeNumber}`);
	}
	titleParts.push(displayName);
	const title = titleParts.join(" ");

	const clampedPercent = Math.max(0, Math.min(100, (progressPercent ?? 0) * 100));
	const remainingLabel =
		countdownSeconds != null && countdownSeconds > 0 ? `Playing in ${Math.ceil(countdownSeconds)}s` : "Play now";

	return (
		<div className="flex items-center" onClick={(e) => e.stopPropagation()}>
			<Image
				type={ImageType.Thumbnail}
				asset={thumbnailImage}
				alt={displayName}
				className="h-32 shrink-0 rounded-r-none object-cover"
			/>

			<div className={cn("h-32 rounded-md rounded-l-none bg-black p-3 pl-5 shadow-lg w-[20em]")}>
				<div className="flex flex-col h-full gap-3">
					<div className="flex-1 min-h-0 overflow-hidden">
						<p className="truncate text-sm font-semibold text-white">{title}</p>
						{description && <p className="text-xs text-white/70 line-clamp-3">{description}</p>}
					</div>
					{onPlay && (
						<div className="flex gap-2">
							<button
								type="button"
								onClick={(e) => {
									e.stopPropagation();
									onPlay();
								}}
								className="relative overflow-hidden rounded bg-white/70 px-3 py-1 text-xs font-medium text-black transition-colors hover:underline"
							>
								{clampedPercent > 0 && (
									<div className="pointer-events-none absolute inset-0">
										<div
											className="h-full bg-white/90  transition-[width] duration-100 ease-linear"
											style={{ width: `${clampedPercent}%` }}
										/>
									</div>
								)}
								<span className="relative z-10">{remainingLabel}</span>
							</button>
							{onCancel && (
								<button
									type="button"
									onClick={(e) => {
										e.stopPropagation();
										onCancel();
									}}
									className="rounded px-3 py-1 text-xs font-medium text-white/60 transition-colors hover:underline"
								>
									Cancel
								</button>
							)}
						</div>
					)}
				</div>
			</div>
		</div>
	);
};
