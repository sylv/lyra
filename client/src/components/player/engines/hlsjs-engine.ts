import type Hls from "hls.js";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { setPlayerLoading } from "../player-state";
import { videoState } from "../video-state";
import type { PlaybackEngine, ResumeConfig } from ".";

type HlsConstructor = typeof Hls;
type ServerTracks = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["tracks"]>;
type Recommendations = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["recommendedTracks"]>;

export const createHlsJsEngine = (
	HlsClass: HlsConstructor,
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

	let hasStartedLoading = false;
	const startLoadAt = (startPosition: number) => {
		if (hasStartedLoading) return;
		hasStartedLoading = true;
		hls.startLoad(Number.isFinite(startPosition) ? startPosition : -1);
	};

	const serverTrackByManifestIndex = (type: "AUDIO" | "SUBTITLE", manifestIndex: number) =>
		serverTracks.find((t) => t.trackType === type && t.manifestIndex === manifestIndex);

	const syncAudioTracks = () => {
		const tracks = hls.audioTracks.map((_track, id) => {
			const serverTrack = serverTrackByManifestIndex("AUDIO", id);
			return { id, label: serverTrack?.displayName ?? `Audio ${id + 1}` };
		});
		videoState.setState({
			audioTrackOptions: tracks,
			selectedAudioTrackId: hls.audioTrack >= 0 ? hls.audioTrack : null,
		});
	};

	const syncSubtitleTracks = () => {
		const tracks = hls.subtitleTracks.map((_track, id) => {
			const serverTrack = serverTrackByManifestIndex("SUBTITLE", id);
			return { id, label: serverTrack?.displayName ?? `Subtitle ${id + 1}` };
		});
		videoState.setState({
			subtitleTrackOptions: tracks,
			selectedSubtitleTrackId: tracks.length > 0 ? (hls.subtitleTrack >= 0 ? hls.subtitleTrack : -1) : null,
		});
	};

	const applyRecommendations = () => {
		for (const rec of recommendations) {
			if (rec.trackType === "AUDIO" && rec.enabled) {
				hls.audioTrack = rec.manifestIndex;
			}
		}
		const enabledSub = recommendations.find((r) => r.trackType === "SUBTITLE" && r.enabled);
		if (enabledSub) {
			hls.subtitleDisplay = true;
			hls.subtitleTrack = enabledSub.manifestIndex;
		} else {
			hls.subtitleDisplay = false;
			hls.subtitleTrack = -1;
		}
	};

	const hls = new HlsClass({ autoStartLoad: false });

	hls.on(HlsClass.Events.ERROR, (event, data) => {
		console.error("HLS error:", event, data);
		if (data.fatal) {
			videoState.setState({ errorMessage: `${data.type}: ${data.reason}` });
			setPlayerLoading(false);
		}
	});

	hls.on(HlsClass.Events.MANIFEST_PARSED, () => {
		syncAudioTracks();
		syncSubtitleTracks();
		applyRecommendations();

		if (!hasResumableWatchProgress) {
			startLoadAt(-1);
			return;
		}

		const levelDurations = hls.levels
			.map((level) => level.details?.totalduration)
			.filter((value): value is number => typeof value === "number" && Number.isFinite(value) && value > 0);
		const durationSeconds = levelDurations[0] ?? runtimeDurationSeconds;
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
						startLoadAt(resumePosition);
					},
					cancelResumePrompt: () => {
						videoState.setState({ resumePromptPosition: null, confirmResumePrompt: null, cancelResumePrompt: null });
						startLoadAt(-1);
					},
				});
				return;
			}

			if (videoRef.current) videoRef.current.currentTime = resumePosition;
			startLoadAt(resumePosition);
			return;
		}

		startLoadAt(-1);
	});

	hls.on(HlsClass.Events.AUDIO_TRACKS_UPDATED, syncAudioTracks);
	hls.on(HlsClass.Events.AUDIO_TRACK_SWITCHED, (_event, data) => {
		if (typeof data.id === "number") {
			videoState.setState({ selectedAudioTrackId: data.id });
		}
	});
	hls.on(HlsClass.Events.SUBTITLE_TRACKS_UPDATED, syncSubtitleTracks);
	hls.on(HlsClass.Events.SUBTITLE_TRACKS_CLEARED, () => {
		videoState.setState({ subtitleTrackOptions: [], selectedSubtitleTrackId: null });
	});
	hls.on(HlsClass.Events.SUBTITLE_TRACK_SWITCH, (_event, data) => {
		if (typeof data.id === "number") {
			videoState.setState({ selectedSubtitleTrackId: data.id });
		} else if (hls.subtitleTracks.length > 0) {
			videoState.setState({ selectedSubtitleTrackId: -1 });
		} else {
			videoState.setState({ selectedSubtitleTrackId: null });
		}
	});

	hls.loadSource(hlsUrl);
	hls.attachMedia(video);

	return {
		setAudioTrack(id) {
			hls.audioTrack = id;
		},
		setSubtitleTrack(id) {
			hls.subtitleTrack = id;
		},
		setSubtitleDisplay(enabled) {
			hls.subtitleDisplay = enabled;
		},
		destroy() {
			hls.destroy();
		},
	};
};
