import type { ItemPlaybackQuery } from "../../@generated/gql/graphql";
import { setPlayerControls, setPlayerLoading, setPlayerState } from "./player-context";

type ServerTracks = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["tracks"]>;
type Recommendations = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["recommendedTracks"]>;

export interface ResumeConfig {
	initialPositionSeconds: number | null;
	watchProgressPercent: number | null | undefined;
	runtimeDurationSeconds: number | null;
	shouldPromptResume: boolean;
	pauseAfterInitialSeek: boolean;
	videoRef: React.RefObject<HTMLVideoElement | null>;
}

export interface PlayerController {
	setAudioTrack(id: number): void;
	setSubtitleTrack(id: number): void;
	setSubtitleDisplay(enabled: boolean): void;
	destroy(): void;
}

export const createHlsPlayer = async (
	video: HTMLVideoElement,
	hlsUrl: string,
	serverTracks: ServerTracks,
	recommendations: Recommendations,
	resumeConfig: ResumeConfig,
): Promise<PlayerController | null> => {
	const { default: Hls } = await import("hls.js");

	if (!Hls.isSupported()) {
		setPlayerState({ errorMessage: "Sorry, your browser does not support this video." });
		setPlayerLoading(false);
		return null;
	}

	const { initialPositionSeconds, watchProgressPercent, runtimeDurationSeconds, shouldPromptResume, pauseAfterInitialSeek, videoRef } =
		resumeConfig;

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
		serverTracks.find((track) => track.trackType === type && track.manifestIndex === manifestIndex);

	const syncAudioTracks = () => {
		const tracks = hls.audioTracks.map((_track, id) => {
			const serverTrack = serverTrackByManifestIndex("AUDIO", id);
			return { id, label: serverTrack?.displayName ?? `Audio ${id + 1}` };
		});

		setPlayerState({
			audioTrackOptions: tracks,
			selectedAudioTrackId: hls.audioTrack >= 0 ? hls.audioTrack : null,
		});
	};

	const syncSubtitleTracks = () => {
		const tracks = hls.subtitleTracks.map((_track, id) => {
			const serverTrack = serverTrackByManifestIndex("SUBTITLE", id);
			return { id, label: serverTrack?.displayName ?? `Subtitle ${id + 1}` };
		});

		setPlayerState({
			subtitleTrackOptions: tracks,
			selectedSubtitleTrackId: tracks.length > 0 ? (hls.subtitleTrack >= 0 ? hls.subtitleTrack : -1) : null,
		});
	};

	const applyRecommendations = () => {
		for (const recommendation of recommendations) {
			if (recommendation.trackType === "AUDIO" && recommendation.enabled) {
				hls.audioTrack = recommendation.manifestIndex;
			}
		}

		const enabledSubtitles = recommendations.find(
			(recommendation) => recommendation.trackType === "SUBTITLE" && recommendation.enabled,
		);
		if (enabledSubtitles) {
			hls.subtitleDisplay = true;
			hls.subtitleTrack = enabledSubtitles.manifestIndex;
		} else {
			hls.subtitleDisplay = false;
			hls.subtitleTrack = -1;
		}
	};

	const hls = new Hls({ autoStartLoad: false });

	hls.on(Hls.Events.ERROR, (event, data) => {
		console.error("HLS error:", event, data);
		if (data.fatal) {
			setPlayerState({ errorMessage: `${data.type}: ${data.reason}` });
			setPlayerLoading(false);
		}
	});

	hls.on(Hls.Events.MANIFEST_PARSED, () => {
		syncAudioTracks();
		syncSubtitleTracks();
		applyRecommendations();

		if (
			typeof initialPositionSeconds === "number" &&
			Number.isFinite(initialPositionSeconds) &&
			initialPositionSeconds >= 0
		) {
			if (videoRef.current) {
				videoRef.current.currentTime = initialPositionSeconds;
				if (pauseAfterInitialSeek) videoRef.current.pause();
			}
			startLoadAt(initialPositionSeconds);
			return;
		}

		if (!hasResumableWatchProgress) {
			startLoadAt(-1);
			return;
		}

		const levelDurations = hls.levels
			.map((level) => level.details?.totalduration)
			.filter((value): value is number => typeof value === "number" && Number.isFinite(value) && value > 0);
		const durationSeconds = levelDurations[0] ?? runtimeDurationSeconds;
		const resumePosition = durationSeconds == null ? null : clampResumePosition(durationSeconds);

		if (resumePosition == null) {
			startLoadAt(-1);
			return;
		}

		if (shouldPromptResume) {
			// keep the callbacks in controls state so the dialog can resolve the prompt without prop drilling.
			setPlayerControls({
				resumePromptPosition: resumePosition,
				confirmResumePrompt: () => {
					setPlayerControls({ resumePromptPosition: null, confirmResumePrompt: null, cancelResumePrompt: null });
					if (videoRef.current) videoRef.current.currentTime = resumePosition;
					startLoadAt(resumePosition);
				},
				cancelResumePrompt: () => {
					setPlayerControls({ resumePromptPosition: null, confirmResumePrompt: null, cancelResumePrompt: null });
					startLoadAt(-1);
				},
			});
			return;
		}

		if (videoRef.current) videoRef.current.currentTime = resumePosition;
		startLoadAt(resumePosition);
	});

	hls.on(Hls.Events.AUDIO_TRACKS_UPDATED, syncAudioTracks);
	hls.on(Hls.Events.AUDIO_TRACK_SWITCHED, (_event, data) => {
		if (typeof data.id === "number") {
			setPlayerState({ selectedAudioTrackId: data.id });
		}
	});
	hls.on(Hls.Events.SUBTITLE_TRACKS_UPDATED, syncSubtitleTracks);
	hls.on(Hls.Events.SUBTITLE_TRACKS_CLEARED, () => {
		setPlayerState({ subtitleTrackOptions: [], selectedSubtitleTrackId: null });
	});
	hls.on(Hls.Events.SUBTITLE_TRACK_SWITCH, (_event, data) => {
		if (typeof data.id === "number") {
			setPlayerState({ selectedSubtitleTrackId: data.id });
		} else if (hls.subtitleTracks.length > 0) {
			setPlayerState({ selectedSubtitleTrackId: -1 });
		} else {
			setPlayerState({ selectedSubtitleTrackId: null });
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
