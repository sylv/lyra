import type { FC } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../../ui/dialog";
import { usePlayerContext } from "../player-context";
import { usePlayerRefsContext } from "../player-refs-context";

const formatResumeTimestamp = (seconds: number): string => {
	const safeSeconds = Math.max(0, Math.floor(seconds));
	const hours = Math.floor(safeSeconds / 3600);
	const minutes = Math.floor((safeSeconds % 3600) / 60);
	const remainingSeconds = safeSeconds % 60;
	if (hours > 0)
		return `${hours}:${minutes.toString().padStart(2, "0")}:${remainingSeconds.toString().padStart(2, "0")}`;
	return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
};

export const ResumePromptDialog: FC = () => {
	const resumePromptPosition = usePlayerContext((ctx) => ctx.controls.resumePromptPosition);
	const confirmResumePrompt = usePlayerContext((ctx) => ctx.controls.confirmResumePrompt);
	const cancelResumePrompt = usePlayerContext((ctx) => ctx.controls.cancelResumePrompt);
	const { containerRef } = usePlayerRefsContext();

	return (
		<Dialog
			open={resumePromptPosition != null}
			onOpenChange={(open) => {
				if (!open) cancelResumePrompt?.();
			}}
		>
			<DialogContent
				portalContainer={containerRef.current}
				className="max-w-sm gap-0 overflow-hidden p-0 [&>button.absolute]:hidden"
				onClick={(event) => event.stopPropagation()}
			>
				<DialogHeader className="sr-only">
					<DialogTitle>Choose playback start</DialogTitle>
				</DialogHeader>
				<div className="flex flex-col">
					<button
						type="button"
						className="w-full border-b border-zinc-700/80 bg-zinc-900/95 px-5 py-4 text-left text-sm font-semibold transition-colors hover:bg-zinc-800/95"
						onClick={confirmResumePrompt ?? undefined}
					>
						Resume from {resumePromptPosition == null ? "0:00" : formatResumeTimestamp(resumePromptPosition)}
					</button>
					<button
						type="button"
						className="w-full bg-zinc-900/95 px-5 py-4 text-left text-sm font-semibold transition-colors hover:bg-zinc-800/95"
						onClick={cancelResumePrompt ?? undefined}
					>
						Start from the beginning
					</button>
				</div>
			</DialogContent>
		</Dialog>
	);
};
