/** biome-ignore-all lint/a11y/useMediaCaption: hls will add captions when available */
/** biome-ignore-all lint/a11y/useKeyWithClickEvents: keyboard users would use arrow keys and 0-9 to seek */
import { useState, type FC } from "react";
import { formatPlayerTime } from "../utils";
import { cn } from "../../../lib/utils";

interface PlayerProcessBarProps {
	duration: number;
	currentTime: number;
	bufferedRanges: { start: number; end: number }[];
	onChange: (time: number) => void;
}

export const PlayerProgressBar: FC<PlayerProcessBarProps> = ({ duration, currentTime, bufferedRanges, onChange }) => {
	const [hoverTime, setHoverTime] = useState<number | null>(null);

	const handleProgressClick = (event: React.MouseEvent<HTMLDivElement>) => {
		event.stopPropagation();

		const rect = event.currentTarget.getBoundingClientRect();
		const clickX = event.clientX - rect.left;
		const newTime = (clickX / rect.width) * duration;
		onChange(newTime);
	};

	const handleProgressMouseMove = (event: React.MouseEvent<HTMLDivElement>) => {
		if (!duration) return;

		const rect = event.currentTarget.getBoundingClientRect();
		const hoverX = event.clientX - rect.left;
		const hoverTimeValue = (hoverX / rect.width) * duration;
		setHoverTime(Math.max(0, Math.min(duration, hoverTimeValue)));
	};

	const onMouseLeave = () => {
		setHoverTime(null);
	};

	const progressPercent = duration ? (currentTime / duration) * 100 : 0;

	return (
		<div
			className="py-2 my-2 cursor-pointer"
			onClick={handleProgressClick}
			onMouseMove={handleProgressMouseMove}
			onMouseLeave={onMouseLeave}
			role="slider"
			tabIndex={0}
			aria-label="Seek video"
			aria-valuemin={0}
			aria-valuemax={duration || 100}
			aria-valuenow={currentTime || 0}
		>
			<div className="relative h-1 bg-white/15 group-hover:h-2 transition-all rounded-md">
				<div className="h-full bg-white/80 transition-all rounded-md" style={{ width: `${progressPercent}%` }} />
				{/* Buffered ranges */}
				{bufferedRanges.map((range) => {
					if (!duration) return null;
					const startPercent = (range.start / duration) * 100;
					const widthPercent = ((range.end - range.start) / duration) * 100;
					return (
						<div
							key={`${range.start}-${range.end}`}
							className="h-full absolute top-0 bg-white/15 transition-all"
							style={{
								left: `${startPercent}%`,
								width: `${widthPercent}%`,
							}}
						/>
					);
				})}
				{/* Hover time tooltip */}
				{hoverTime && (
					<div
						className="absolute top-0 bottom-0"
						style={{
							left: `${(hoverTime / (duration || 1)) * 100}%`,
						}}
					>
						<div className={cn("absolute -top-1 bottom-0 w-0.5 shadow-lg z-20 bg-white/40")} />
						<div className={cn("absolute -top-8 bg-black/60 px-2 py-0.5 rounded-lg text-sm -translate-x-1/2")}>
							{formatPlayerTime(hoverTime)}
						</div>
					</div>
				)}
			</div>
		</div>
	);
};
