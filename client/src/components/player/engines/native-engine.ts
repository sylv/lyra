import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { setPlayerLoading } from "../player-state";
import { videoState } from "../video-state";
import type { PlaybackEngine, ResumeConfig } from ".";

type ServerTracks = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["tracks"]>;
type Recommendations = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["recommendedTracks"]>;

export const createNativeEngine = (
	video: HTMLVideoElement,
	hlsUrl: string,
	serverTracks: ServerTracks,
	recommendations: Recommendations,
	resumeConfig: ResumeConfig,
): PlaybackEngine => {
	const { watchProgressPercent, runtimeDurationSeconds, shouldPromptResume, videoRef } = resumeConfig;

	const hasResumableWatchProgress =
		typeof watchProgressPercent === "number" &&
		Number.isFinite(watchProgressPercent) &&
		watchProgressPercent > 0 &&
		watchProgressPercent < 1;
	const safeWatchProgressPercent = hasResumableWatchProgress ? watchProgressPercent : 0;

	const clampResumePosition = (durationSeconds: number) => {
		if (!hasResumableWatchProgress) return null;
		const progress = Math.max(0, Math.min(0.999, safeWatchProgressPercent));
		const maxStart = Math.max(0, durationSeconds - 0.5);
		return Math.max(0, Math.min(progress * durationSeconds, maxStart));
	};

	const serverTrackByManifestIndex = (type: "AUDIO" | "SUBTITLE", manifestIndex: number) =>
		serverTracks.find((t) => t.trackType === type && t.manifestIndex === manifestIndex);

	const onLoadedMetadata = () => {
		// sync audio tracks from native AudioTrackList (non-standard API, not in TS DOM types)
		const audioTrackList = (
			video as HTMLVideoElement & { audioTracks?: { length: number; [i: number]: { enabled: boolean } | undefined } }
		).audioTracks;
		if (audioTrackList) {
			const tracks: Array<{ id: number; label: string }> = [];
			let selectedId: number | null = null;
			for (let i = 0; i < audioTrackList.length; i++) {
				const serverTrack = serverTrackByManifestIndex("AUDIO", i);
				tracks.push({ id: i, label: serverTrack?.displayName ?? `Audio ${i + 1}` });
				if (audioTrackList[i]?.enabled) selectedId = i;
			}
			videoState.setState({ audioTrackOptions: tracks, selectedAudioTrackId: selectedId });
		}

		// sync text (subtitle) tracks
		const textTracks = video.textTracks;
		const subtitleTracks: Array<{ id: number; label: string }> = [];
		for (let i = 0; i < textTracks.length; i++) {
			const serverTrack = serverTrackByManifestIndex("SUBTITLE", i);
			subtitleTracks.push({ id: i, label: serverTrack?.displayName ?? `Subtitle ${i + 1}` });
		}
		videoState.setState({
			subtitleTrackOptions: subtitleTracks,
			selectedSubtitleTrackId: subtitleTracks.length > 0 ? -1 : null,
		});

		// apply recommendations
		if (audioTrackList) {
			for (const rec of recommendations) {
				if (rec.trackType === "AUDIO" && rec.enabled) {
					for (let i = 0; i < audioTrackList.length; i++) {
						const t = audioTrackList[i];
						if (t) t.enabled = i === rec.manifestIndex;
					}
					videoState.setState({ selectedAudioTrackId: rec.manifestIndex });
				}
			}
		}
		const enabledSub = recommendations.find((r) => r.trackType === "SUBTITLE" && r.enabled);
		if (enabledSub) {
			for (let i = 0; i < textTracks.length; i++) {
				const t = textTracks[i];
				if (t) t.mode = i === enabledSub.manifestIndex ? "showing" : "hidden";
			}
			videoState.setState({ selectedSubtitleTrackId: enabledSub.manifestIndex });
		} else {
			for (let i = 0; i < textTracks.length; i++) {
				const t = textTracks[i];
				if (t) t.mode = "hidden";
			}
		}

		// handle resume: use video.duration (available after loadedmetadata), fall back to runtime hint
		const durationSeconds =
			Number.isFinite(video.duration) && video.duration > 0 ? video.duration : runtimeDurationSeconds;
		const resumePosition = durationSeconds == null ? null : clampResumePosition(durationSeconds);

		if (resumePosition != null) {
			if (shouldPromptResume) {
				// store callbacks in videoState so ResumePromptDialog can call them without prop drilling.
				// after either callback fires, it clears itself to prevent double-invocation via onOpenChange.
				videoState.setState({
					resumePromptPosition: resumePosition,
					confirmResumePrompt: () => {
						videoState.setState({ resumePromptPosition: null, confirmResumePrompt: null, cancelResumePrompt: null });
						if (videoRef.current) videoRef.current.currentTime = resumePosition;
					},
					cancelResumePrompt: () => {
						videoState.setState({ resumePromptPosition: null, confirmResumePrompt: null, cancelResumePrompt: null });
					},
				});
				return;
			}

			if (videoRef.current) videoRef.current.currentTime = resumePosition;
		}
	};

	const onError = () => {
		const err = video.error;
		const message = err ? `Media error ${err.code}: ${err.message}` : "An unknown playback error occurred.";
		console.error("Native HLS error:", err);
		videoState.setState({ errorMessage: message });
		setPlayerLoading(false);
	};

	video.addEventListener("loadedmetadata", onLoadedMetadata);
	video.addEventListener("error", onError);
	video.src = hlsUrl;

	return {
		setAudioTrack(id) {
			const audioTrackList = (
				video as HTMLVideoElement & { audioTracks?: { length: number; [i: number]: { enabled: boolean } | undefined } }
			).audioTracks;
			if (!audioTrackList) return;
			for (let i = 0; i < audioTrackList.length; i++) {
				const t = audioTrackList[i];
				if (t) t.enabled = i === id;
			}
			videoState.setState({ selectedAudioTrackId: id });
		},
		setSubtitleTrack(id) {
			const textTracks = video.textTracks;
			for (let i = 0; i < textTracks.length; i++) {
				const t = textTracks[i];
				if (t) t.mode = i === id ? "showing" : "hidden";
			}
			videoState.setState({ selectedSubtitleTrackId: id >= 0 ? id : -1 });
		},
		setSubtitleDisplay(enabled) {
			if (!enabled) {
				const textTracks = video.textTracks;
				for (let i = 0; i < textTracks.length; i++) {
					const t = textTracks[i];
					if (t) t.mode = "hidden";
				}
				videoState.setState({ selectedSubtitleTrackId: -1 });
			}
		},
		destroy() {
			video.removeEventListener("loadedmetadata", onLoadedMetadata);
			video.removeEventListener("error", onError);
			video.removeAttribute("src");
			video.load();
		},
	};
};
