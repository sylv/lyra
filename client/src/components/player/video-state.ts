import { create } from "zustand";

interface VideoState {
	playing: boolean;
	currentTime: number;
	duration: number;
	bufferedRanges: Array<{ start: number; end: number }>;
	videoAspectRatio: number;
	errorMessage: string | null;

	audioTrackOptions: Array<{ id: number; label: string }>;
	selectedAudioTrackId: number | null;
	subtitleTrackOptions: Array<{ id: number; label: string }>;
	selectedSubtitleTrackId: number | null;

	showControls: boolean;
	isSettingsMenuOpen: boolean;
	isControlsInteracting: boolean;

	// non-null while the resume prompt is showing; cleared when a decision is made.
	resumePromptPosition: number | null;
	confirmResumePrompt: (() => void) | null;
	cancelResumePrompt: (() => void) | null;
}

export const videoState = create<VideoState>()(() => ({
	playing: false,
	currentTime: 0,
	duration: 0,
	bufferedRanges: [],
	videoAspectRatio: 16 / 9,
	errorMessage: null,

	audioTrackOptions: [],
	selectedAudioTrackId: null,
	subtitleTrackOptions: [],
	selectedSubtitleTrackId: null,

	showControls: true,
	isSettingsMenuOpen: false,
	isControlsInteracting: false,

	resumePromptPosition: null,
	confirmResumePrompt: null,
	cancelResumePrompt: null,
}));
