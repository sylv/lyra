import { cn } from "../../../lib/utils";

interface SkipIntroButtonProps {
	progressPercent: number;
	onSkip: () => void;
}

export const SkipIntroButton = ({ progressPercent, onSkip }: SkipIntroButtonProps) => {
	const clampedProgressPercent = Math.max(0, Math.min(100, progressPercent * 100));

	return (
		<button
			type="button"
			onClick={(event) => {
				// or else the click will pause the player by propogating up and being considered
				// a click on the video.
				event.stopPropagation();
				onSkip();
			}}
			className={cn(
				"relative overflow-hidden rounded-md bg-white/60 px-3 py-2 text-left text-black shadow-lg backdrop-blur-sm transition-colors hover:bg-white/50",
			)}
		>
			<div className="pointer-events-none absolute inset-0">
				<div
					className="h-full bg-white/30 transition-[width] duration-300 ease-linear"
					style={{
						width: `${clampedProgressPercent}%`,
					}}
				/>
			</div>

			<div className="relative z-10 flex items-center gap-3 px-6">
				<span className="text-sm font-semibold">Skip Intro</span>
			</div>
		</button>
	);
};
