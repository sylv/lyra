/* oxlint-disable jsx_a11y/media-has-caption */
import { useMutation } from "@apollo/client/react";
import { useEffect, useRef, type FC } from "react";
import { type FragmentType } from "../../@generated/gql";
import type { ItemPlaybackQuery } from "../../@generated/gql/graphql";
import { createHlsPlayer } from "./hls";
import {
	playerContext,
	resetPlayerControls,
	resetPlayerState,
	setPlayerActions,
	setPlayerLoading,
	setPlayerMuted,
	setPlayerSnapshot,
	setPlayerState,
	setPlayerVolume,
} from "./player-context";
import { UpdateWatchState } from "./player-queries";
import { PlayerTimelinePreviewSheetFragment } from "./components/player-progress-bar";
import { usePlayerRefsContext } from "./player-refs-context";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

interface PlayerVideoProps {
	currentMedia: CurrentMedia | null;
	autoplay: boolean;
	shouldPromptResume: boolean;
}

export const PlayerVideo: FC<PlayerVideoProps> = ({ currentMedia, autoplay, shouldPromptResume }) => {
	const { videoRef, controllerRef } = usePlayerRefsContext();
	const [updateWatchProgress] = useMutation(UpdateWatchState);
	const currentMediaId = currentMedia?.id ?? null;
	const currentFileId = currentMedia?.file?.id ?? null;
	const snapshotUpdateRef = useRef<{ mediaId: string | null; lastPosition: number | null; lastUpdatedAt: number }>({
		mediaId: null,
		lastPosition: null,
		lastUpdatedAt: 0,
	});
	const watchProgressRef = useRef<{ mediaId: string | null; fileId: string | null; lastProgressPercent: number | null }>({
		mediaId: null,
		fileId: null,
		lastProgressPercent: null,
	});

	useEffect(() => {
		const video = videoRef.current;
		if (!video) return;

		const togglePlaying = () => {
			if (video.paused) {
				video.play().catch(() => undefined);
			} else {
				video.pause();
			}
		};

		const seekTo = (time: number) => {
			video.currentTime = time;
		};

		const seekBy = (deltaSeconds: number) => {
			const cachedDuration = playerContext.getState().state.duration;
			const duration = Number.isFinite(video.duration) && video.duration > 0 ? video.duration : cachedDuration;
			const nextTime = video.currentTime + deltaSeconds;
			video.currentTime = Math.max(0, duration > 0 ? Math.min(duration, nextTime) : nextTime);
		};

		const toggleMute = () => {
			const nextMuted = !playerContext.getState().preferences.isMuted;
			setPlayerMuted(nextMuted);
			video.muted = nextMuted;
		};

		const setVolume = (volume: number) => {
			setPlayerVolume(volume);
			video.volume = volume;
			if (volume > 0 && playerContext.getState().preferences.isMuted) {
				setPlayerMuted(false);
				video.muted = false;
			}
		};

		const setAudioTrack = (trackId: number) => {
			controllerRef.current?.setAudioTrack(trackId);
		};

		const setSubtitleTrack = (trackId: number) => {
			controllerRef.current?.setSubtitleTrack(trackId);
		};

		const setSubtitleDisplay = (enabled: boolean) => {
			controllerRef.current?.setSubtitleDisplay(enabled);
		};

		setPlayerActions({
			togglePlaying,
			seekBy,
			seekTo,
			toggleMute,
			setVolume,
			setAudioTrack,
			setSubtitleTrack,
			setSubtitleDisplay,
		});
	}, [videoRef, controllerRef]);

	useEffect(() => {
		if (!videoRef.current) return;
		videoRef.current.volume = playerContext.getState().preferences.volume;
		videoRef.current.muted = playerContext.getState().preferences.isMuted;
	}, [videoRef]);

	useEffect(() => {
		if (!videoRef.current || !currentMedia) return;
		const video = videoRef.current;

		if (controllerRef.current) {
			controllerRef.current.destroy();
			controllerRef.current = null;
		}

		if (!autoplay) {
			video.pause();
			setPlayerState({ playing: false });
		}

		if (!currentMedia.file) {
			video.pause();
			setPlayerState({ errorMessage: "Sorry, this item is unavailable" });
			setPlayerLoading(false);
			return;
		}

		setPlayerState({ errorMessage: null });
		setPlayerLoading(true);

		const hlsUrl = `/api/hls/stream/${currentMedia.file.id}/master.m3u8`;
		const initialPositionSeconds = playerContext.getState().state.pendingInitialPosition;
		const watchProgressPercent = currentMedia.watchProgress?.completed
			? null
			: currentMedia.watchProgress?.progressPercent;
		const runtimeMinutes = currentMedia.properties.runtimeMinutes;
		const runtimeDurationSeconds =
			typeof runtimeMinutes === "number" && Number.isFinite(runtimeMinutes) && runtimeMinutes > 0
				? runtimeMinutes * 60
				: null;

		let active = true;

		createHlsPlayer(video, hlsUrl, currentMedia.file.tracks ?? [], currentMedia.file.recommendedTracks ?? [], {
			initialPositionSeconds,
			watchProgressPercent,
			runtimeDurationSeconds,
			shouldPromptResume,
			pauseAfterInitialSeek: initialPositionSeconds != null,
			videoRef,
		}).then((controller) => {
			if (!active) {
				controller?.destroy();
				return;
			}
			controllerRef.current = controller;
			if (initialPositionSeconds != null) {
				setPlayerState({ pendingInitialPosition: null, playing: false });
			}
		});

		return () => {
			active = false;
			resetPlayerState({
				autoplay: playerContext.getState().state.autoplay,
				shouldPromptResume: false,
				isFullscreen: playerContext.getState().state.isFullscreen,
			});
			resetPlayerControls();
			controllerRef.current?.destroy();
			controllerRef.current = null;
		};
	}, [autoplay, currentMediaId, currentFileId, controllerRef, shouldPromptResume, videoRef]);

	useEffect(() => {
		const video = videoRef.current;
		if (!video) return;

		// keep a persisted local snapshot of the active item so reloads can restore the exact position.
		const syncPlayerSnapshot = (force = false) => {
			if (!currentMedia) return;
			if (playerContext.getState().currentItemId !== currentMedia.id) return;

			const position = Number.isFinite(video.currentTime) && video.currentTime > 0 ? video.currentTime : 0;
			const now = Date.now();
			const snapshotState = snapshotUpdateRef.current;

			if (snapshotState.mediaId !== currentMedia.id) {
				snapshotUpdateRef.current = {
					mediaId: currentMedia.id,
					lastPosition: null,
					lastUpdatedAt: 0,
				};
			}

			const nextSnapshotState = snapshotUpdateRef.current;
			if (!force) {
				const positionDelta =
					nextSnapshotState.lastPosition == null ? Number.POSITIVE_INFINITY : Math.abs(position - nextSnapshotState.lastPosition);
				if (positionDelta < 1 && now - nextSnapshotState.lastUpdatedAt < 1_000) return;
			}

			setPlayerSnapshot({
				currentItemId: currentMedia.id,
				position,
			});
			snapshotUpdateRef.current = {
				mediaId: currentMedia.id,
				lastPosition: position,
				lastUpdatedAt: now,
			};
		};

		const syncWatchProgress = () => {
			if (!currentMedia?.file || video.duration <= 0) return;

			const progressPercent = video.currentTime / video.duration;
			if (!Number.isFinite(progressPercent)) return;

			if (
				watchProgressRef.current.mediaId !== currentMedia.id ||
				watchProgressRef.current.fileId !== currentMedia.file.id
			) {
				watchProgressRef.current = {
					mediaId: currentMedia.id,
					fileId: currentMedia.file.id,
					lastProgressPercent: null,
				};
			}

			if (watchProgressRef.current.lastProgressPercent === progressPercent) return;
			watchProgressRef.current.lastProgressPercent = progressPercent;

			updateWatchProgress({
				variables: {
					fileId: currentMedia.file.id,
					progressPercent,
				},
			}).catch((err: unknown) => {
				console.error("failed to update watch state", err);
			});
		};

		const updateBufferedRanges = () => {
			const ranges: Array<{ start: number; end: number }> = [];
			for (let i = 0; i < video.buffered.length; i++) {
				ranges.push({ start: video.buffered.start(i), end: video.buffered.end(i) });
			}
			setPlayerState({ bufferedRanges: ranges });
		};

		const updatePlaybackState = () => {
			setPlayerState({
				playing: !video.paused,
				currentTime: video.currentTime,
				duration: video.duration,
				...(!video.paused ? { ended: false } : {}),
			});
			setPlayerVolume(video.volume);
			setPlayerMuted(video.muted);
			updateBufferedRanges();
			syncPlayerSnapshot();
		};

		const handleLoadedMetadata = () => {
			if (video.videoWidth <= 0 || video.videoHeight <= 0) return;
			setPlayerState({ videoAspectRatio: video.videoWidth / video.videoHeight });
			updatePlaybackState();
		};

		const handleEnded = () => {
			setPlayerState({ ended: true, playing: false });
		};

		const handleLoadStart = () => {
			setPlayerLoading(true);
			setPlayerState({
				currentTime: 0,
				duration: 0,
				bufferedRanges: [],
				ended: false,
				upNextDismissed: false,
				upNextCountdownCancelled: false,
				isUpNextActive: false,
			});
		};

		const handleCanPlay = () => setPlayerLoading(false);
		const handleWaiting = () => setPlayerLoading(true);
		const handlePlaying = () => setPlayerLoading(false);
		const handleLoadedData = () => setPlayerLoading(false);

		let lastUpdated = 0;
		const handleTimeUpdate = () => {
			const now = Date.now();
			if (now - lastUpdated >= 300) {
				lastUpdated = now;
				updatePlaybackState();
			}
		};

		const handleSeeked = () => {
			updatePlaybackState();
			syncPlayerSnapshot(true);
			syncWatchProgress();
		};

		const watchProgressInterval = window.setInterval(syncWatchProgress, 5_000);

		video.addEventListener("timeupdate", handleTimeUpdate);
		video.addEventListener("play", updatePlaybackState);
		video.addEventListener("pause", updatePlaybackState);
		video.addEventListener("loadedmetadata", handleLoadedMetadata);
		video.addEventListener("volumechange", updatePlaybackState);
		video.addEventListener("loadstart", handleLoadStart);
		video.addEventListener("canplay", handleCanPlay);
		video.addEventListener("waiting", handleWaiting);
		video.addEventListener("playing", handlePlaying);
		video.addEventListener("loadeddata", handleLoadedData);
		video.addEventListener("ended", handleEnded);
		video.addEventListener("seeked", handleSeeked);

		return () => {
			syncPlayerSnapshot(true);
			window.clearInterval(watchProgressInterval);
			video.removeEventListener("timeupdate", handleTimeUpdate);
			video.removeEventListener("play", updatePlaybackState);
			video.removeEventListener("pause", updatePlaybackState);
			video.removeEventListener("loadedmetadata", handleLoadedMetadata);
			video.removeEventListener("volumechange", updatePlaybackState);
			video.removeEventListener("loadstart", handleLoadStart);
			video.removeEventListener("canplay", handleCanPlay);
			video.removeEventListener("waiting", handleWaiting);
			video.removeEventListener("playing", handlePlaying);
			video.removeEventListener("loadeddata", handleLoadedData);
			video.removeEventListener("ended", handleEnded);
			video.removeEventListener("seeked", handleSeeked);
		};
	}, [currentMediaId, currentFileId, updateWatchProgress, videoRef]);

	const autoplayEnabled = playerContext.getState().state.autoplay;
	const isFullscreen = playerContext.getState().state.isFullscreen;

	return (
		<video
			ref={videoRef}
			className={isFullscreen ? "block h-full w-full bg-black object-contain outline-none" : "block h-full w-full rounded bg-black object-contain outline-none"}
			autoPlay={autoplayEnabled}
			controls={false}
			disablePictureInPicture
		/>
	);
};

export const getTimelinePreviewSheets = (
	currentMedia: CurrentMedia | null,
): FragmentType<typeof PlayerTimelinePreviewSheetFragment>[] => {
	return Array.isArray(currentMedia?.file?.timelinePreview) ? currentMedia.file.timelinePreview : [];
};
