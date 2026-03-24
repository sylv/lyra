import { useEffect } from "react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { usePlayerContext } from "../player-context";
import { createPlaybackEngine } from "../engines";
import { setPlayerLoading } from "../player-state";
import { videoState } from "../video-state";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

interface PlaybackLifecycleOptions {
	shouldPromptResume: boolean;
	autoplay: boolean;
}

export const usePlaybackLifecycle = (
	currentMedia: CurrentMedia | null,
	{ shouldPromptResume, autoplay }: PlaybackLifecycleOptions,
) => {
	const { videoRef, engineRef, surfaceRef } = usePlayerContext();

	// focus the surface when media changes so keyboard shortcuts work immediately
	useEffect(() => {
		surfaceRef.current?.focus();
	}, [currentMedia?.id, surfaceRef]);

	// when not autoplaying a new item, ensure the video is paused
	useEffect(() => {
		if (autoplay) return;
		videoRef.current?.pause();
		videoState.setState({ playing: false });
	}, [autoplay, currentMedia?.id, videoRef]);

	useEffect(() => {
		if (!videoRef.current || !currentMedia) return;

		if (engineRef.current != null) {
			engineRef.current.destroy();
			engineRef.current = null;
		}

		if (!currentMedia.file) {
			videoRef.current.pause();
			videoState.setState({ errorMessage: "Sorry, this item is unavailable" });
			setPlayerLoading(false);
			return;
		}

		videoState.setState({ errorMessage: null });
		setPlayerLoading(true);

		const hlsUrl = `/api/hls/stream/${currentMedia.file.id}/master.m3u8`;
		const watchProgressPercent = currentMedia.watchProgress?.completed
			? null
			: currentMedia.watchProgress?.progressPercent;
		const runtimeMinutes = currentMedia.properties.runtimeMinutes;
		const runtimeDurationSeconds =
			typeof runtimeMinutes === "number" && Number.isFinite(runtimeMinutes) && runtimeMinutes > 0
				? runtimeMinutes * 60
				: null;

		const serverTracks = currentMedia.file.tracks ?? [];
		const recommendations = currentMedia.file.recommendedTracks ?? [];

		const video = videoRef.current;
		let active = true;

		createPlaybackEngine(video, hlsUrl, serverTracks, recommendations, {
			watchProgressPercent,
			runtimeDurationSeconds,
			shouldPromptResume,
			videoRef,
		}).then((engine) => {
			if (!active) {
				engine?.destroy();
				return;
			}
			engineRef.current = engine;
		});

		return () => {
			active = false;
			// reset per-media state so stale track options / prompts don't flash during the next item's load
			videoState.setState({
				audioTrackOptions: [],
				selectedAudioTrackId: null,
				subtitleTrackOptions: [],
				selectedSubtitleTrackId: null,
				isSettingsMenuOpen: false,
				ended: false,
				upNextDismissed: false,
				upNextCountdownCancelled: false,
				isUpNextActive: false,
				isItemCardOpen: false,
				resumePromptPosition: null,
				confirmResumePrompt: null,
				cancelResumePrompt: null,
			});
			engineRef.current?.destroy();
			engineRef.current = null;
		};
	}, [currentMedia?.id, shouldPromptResume, videoRef, engineRef, surfaceRef]);
};
