import type { FC } from "react";
import { useStore } from "zustand/react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../../ui/dialog";
import { usePlayerContext } from "../player-context";
import { videoState } from "../video-state";

const formatResumeTimestamp = (seconds: number): string => {
	const safeSeconds = Math.max(0, Math.floor(seconds));
	const hours = Math.floor(safeSeconds / 3600);
	const minutes = Math.floor((safeSeconds % 3600) / 60);
	const remainingSeconds = safeSeconds % 60;
	if (hours > 0) {
		return `${hours}:${minutes.toString().padStart(2, "0")}:${remainingSeconds.toString().padStart(2, "0")}`;
	}
	return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
};

export const ResumePromptDialog: FC = () => {
	const resumePromptPosition = useStore(videoState, (s) => s.resumePromptPosition);
	const confirmResumePrompt = useStore(videoState, (s) => s.confirmResumePrompt);
	const cancelResumePrompt = useStore(videoState, (s) => s.cancelResumePrompt);
	const { containerRef } = usePlayerContext();

	return (
		<Dialog
			open={resumePromptPosition != null}
			onOpenChange={(open) => {
				if (open) return;
				// after confirm/cancel clears the callbacks, this becomes a no-op
				cancelResumePrompt?.();
			}}
		>
			<DialogContent
				portalContainer={containerRef.current}
				className="max-w-sm p-0 gap-0 overflow-hidden [&>button.absolute]:hidden"
				onClick={(event) => {
					event.stopPropagation();
				}}
			>
				<DialogHeader className="sr-only">
					<DialogTitle>Choose playback start</DialogTitle>
				</DialogHeader>
				<div className="flex flex-col">
					<button
						type="button"
						className="w-full px-5 py-4 text-left text-sm font-semibold transition-colors bg-zinc-900/95 hover:bg-zinc-800/95 border-b border-zinc-700/80"
						onClick={confirmResumePrompt ?? undefined}
					>
						Resume from {resumePromptPosition == null ? "0:00" : formatResumeTimestamp(resumePromptPosition)}
					</button>
					<button
						type="button"
						className="w-full px-5 py-4 text-left text-sm font-semibold transition-colors bg-zinc-900/95 hover:bg-zinc-800/95"
						onClick={cancelResumePrompt ?? undefined}
					>
						Start from the beginning
					</button>
				</div>
			</DialogContent>
		</Dialog>
	);
};
