/* oxlint-disable jsx_a11y/media-has-caption */
import { useMutation, useSubscription } from "@apollo/client/react";
import { useEffect, useRef, useState, type FC } from "react";
import { type FragmentType, unmask } from "../../@generated/gql";
import {
	type ItemPlaybackQuery,
	WatchSessionActionKind,
	WatchSessionIntent,
} from "../../@generated/gql/graphql";
import { createHlsPlayer } from "./hls";
import {
	playerContext,
	resetPlayerControls,
	resetPlayerState,
	setPlayerActions,
	setPlayerLoading,
	setPlayerMedia,
	setPlayerMuted,
	setPlayerSnapshot,
	setPlayerState,
	setPlayerVolume,
	setPlayerWatchSession,
	usePlayerContext,
} from "./player-context";
import {
	UpdateWatchState,
	WatchSessionAction,
	WatchSessionBeaconFragment,
	WatchSessionBeacons,
	WatchSessionHeartbeat,
} from "./player-queries";
import { PlayerTimelinePreviewSheetFragment } from "./components/player-progress-bar";
import { usePlayerRefsContext } from "./player-refs-context";
import {
	applyWatchSessionBeacon,
	createLocalWatchSessionId,
	getWatchSessionState,
} from "./watch-session";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

interface PlayerVideoProps {
	currentMedia: CurrentMedia | null;
	autoplay: boolean;
	shouldPromptResume: boolean;
}

export const PlayerVideo: FC<PlayerVideoProps> = ({ currentMedia, autoplay, shouldPromptResume }) => {
	const { videoRef, controllerRef } = usePlayerRefsContext();
	const [updateWatchProgress] = useMutation(UpdateWatchState);
	const [watchSessionHeartbeat] = useMutation(WatchSessionHeartbeat);
	const [watchSessionAction] = useMutation(WatchSessionAction);
	const currentMediaId = currentMedia?.id ?? null;
	const currentFileId = currentMedia?.file?.id ?? null;
	const watchSession = usePlayerContext((ctx) => ctx.watchSession);
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
	const sessionControlledSwitchRef = useRef<string | null>(null);
	const pendingActionRef = useRef<Promise<unknown> | null>(null);
	const heartbeatRef = useRef<(() => void) | null>(null);
	const [isWatchSessionRegistered, setIsWatchSessionRegistered] = useState(false);

	useSubscription(WatchSessionBeacons, {
		variables: {
			sessionId: watchSession.sessionId ?? "",
			playerId: watchSession.playerId ?? "",
		},
		skip: !watchSession.sessionId || !watchSession.playerId || !isWatchSessionRegistered,
		onData: ({ data }) => {
			const beacon = data.data?.watchSessionBeacons;
			if (!beacon) return;
			applyWatchSessionBeacon(unmask(WatchSessionBeaconFragment, beacon));
		},
	});

	useEffect(() => {
		setIsWatchSessionRegistered(false);
	}, [watchSession.playerId, watchSession.sessionId]);

	useEffect(() => {
		const video = videoRef.current;
		if (!video) return;

		const sendAction = async (
			kind: WatchSessionActionKind,
			fields: Partial<{ positionMs: number; nodeId: string; targetPlayerId: string }> = {},
		) => {
			const sessionState = getWatchSessionState();
			if (!sessionState.sessionId || !sessionState.playerId) return null;

			const request = watchSessionAction({
				variables: {
					input: {
						sessionId: sessionState.sessionId,
						playerId: sessionState.playerId,
						kind,
						positionMs: fields.positionMs ?? null,
						nodeId: fields.nodeId ?? null,
						targetPlayerId: fields.targetPlayerId ?? null,
					},
				},
			})
				.then((result) => {
					const beacon = result.data?.watchSessionAction;
					if (beacon) {
						applyWatchSessionBeacon(unmask(WatchSessionBeaconFragment, beacon));
					}
					return beacon ?? null;
				})
				.catch((error: unknown) => {
					const sessionMode = getWatchSessionState().mode;
					if (sessionMode === "SYNCED") {
						setPlayerWatchSession({ connectionWarning: "Watch session connection lost" });
					}
					throw error;
				});
			pendingActionRef.current = request;
			return request;
		};

		const togglePlaying = () => {
			const sessionState = getWatchSessionState();
			const positionMs = Math.max(0, Math.round(video.currentTime * 1000));
			if (sessionState.mode === "SYNCED") {
				void sendAction(video.paused ? WatchSessionActionKind.Play : WatchSessionActionKind.Pause, { positionMs });
				return;
			}

			if (video.paused) {
				video.play().catch(() => undefined);
				void sendAction(WatchSessionActionKind.Play, { positionMs });
			} else {
				video.pause();
				void sendAction(WatchSessionActionKind.Pause, { positionMs });
			}
		};

		const seekTo = (time: number) => {
			const target = Math.max(0, time);
			const targetMs = Math.round(target * 1000);
			if (getWatchSessionState().mode === "SYNCED") {
				void sendAction(WatchSessionActionKind.Seek, { positionMs: targetMs });
				return;
			}

			video.currentTime = target;
			void sendAction(WatchSessionActionKind.Seek, { positionMs: targetMs });
		};

		const seekBy = (deltaSeconds: number) => {
			const cachedDuration = playerContext.getState().state.duration;
			const duration = Number.isFinite(video.duration) && video.duration > 0 ? video.duration : cachedDuration;
			const nextTime = video.currentTime + deltaSeconds;
			const target = Math.max(0, duration > 0 ? Math.min(duration, nextTime) : nextTime);
			const targetMs = Math.round(target * 1000);
			if (getWatchSessionState().mode === "SYNCED") {
				void sendAction(WatchSessionActionKind.Seek, { positionMs: targetMs });
				return;
			}

			video.currentTime = target;
			void sendAction(WatchSessionActionKind.Seek, { positionMs: targetMs });
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

		const switchItem = (itemId: string) => {
			if (getWatchSessionState().mode === "SYNCED") {
				void sendAction(WatchSessionActionKind.SwitchItem, { nodeId: itemId });
				return;
			}

			sessionControlledSwitchRef.current = itemId;
			setPlayerMedia(itemId, true);
			void sendAction(WatchSessionActionKind.SwitchItem, { nodeId: itemId });
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
			switchItem,
		});
	}, [controllerRef, videoRef, watchSessionAction]);

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
		if (!currentMedia?.file) return;
		if (watchSession.sessionId && watchSession.playerId) return;

		const pendingSessionId = watchSession.pendingSessionId;
		const pendingNodeId = watchSession.pendingNodeId;
		const shouldJoin = pendingSessionId != null && pendingNodeId === currentMedia.id;
		if (pendingSessionId && !shouldJoin) return;

		setPlayerWatchSession({
			sessionId: shouldJoin ? pendingSessionId : createLocalWatchSessionId(),
			playerId: createLocalWatchSessionId(),
			nodeId: currentMedia.id,
			fileId: currentMedia.file.id,
			mode: "ADVISORY",
			intent: playerContext.getState().state.playing ? "PLAYING" : "PAUSED",
			effectiveState: playerContext.getState().state.playing ? "PLAYING" : "PAUSED",
			basePositionMs: Math.max(0, Math.round((videoRef.current?.currentTime ?? 0) * 1000)),
			baseTimeMs: Date.now(),
			players: [],
			lastContactAt: null,
			connectionWarning: null,
			pendingSessionId: null,
			pendingNodeId: null,
		});
	}, [
		currentMedia?.file,
		currentMedia?.id,
		currentMedia?.file?.id,
		videoRef,
		watchSession.pendingNodeId,
		watchSession.pendingSessionId,
		watchSession.playerId,
		watchSession.sessionId,
	]);

	useEffect(() => {
		if (watchSession.mode === "SYNCED") return;
		if (!currentMediaId || !watchSession.sessionId || !watchSession.playerId) return;
		if (watchSession.pendingSessionId) return;
		if (sessionControlledSwitchRef.current === currentMediaId) {
			sessionControlledSwitchRef.current = null;
			return;
		}
		if (!watchSession.nodeId || watchSession.nodeId === currentMediaId) return;

		void watchSessionAction({
			variables: {
				input: {
					sessionId: watchSession.sessionId,
					playerId: watchSession.playerId,
					kind: WatchSessionActionKind.SwitchItem,
					positionMs: null,
					nodeId: currentMediaId,
					targetPlayerId: null,
				},
			},
		})
			.then((result) => {
				const beacon = result.data?.watchSessionAction;
				if (beacon) {
					applyWatchSessionBeacon(unmask(WatchSessionBeaconFragment, beacon));
				}
			})
			.catch((error) => {
				console.error("failed to switch watch session item", error);
			});
	}, [
		currentMediaId,
		watchSession.nodeId,
		watchSession.pendingSessionId,
		watchSession.playerId,
		watchSession.sessionId,
		watchSession.mode,
		watchSessionAction,
	]);

	useEffect(() => {
		const video = videoRef.current;
		if (!video || watchSession.mode !== "SYNCED") {
			if (video) {
				video.playbackRate = 1;
			}
			return;
		}
		if (watchSession.basePositionMs == null || watchSession.baseTimeMs == null || !watchSession.nodeId) return;

		if (watchSession.nodeId !== currentMediaId) {
			sessionControlledSwitchRef.current = watchSession.nodeId;
			setPlayerState({
				pendingInitialPosition: watchSession.basePositionMs / 1000,
				autoplay: false,
				shouldPromptResume: false,
			});
			setPlayerMedia(watchSession.nodeId, false);
			return;
		}

		const targetSeconds =
			watchSession.effectiveState === "PLAYING"
				? Math.max(0, (watchSession.basePositionMs + Math.max(0, Date.now() - watchSession.baseTimeMs)) / 1000)
				: Math.max(0, watchSession.basePositionMs / 1000);
		const driftSeconds = targetSeconds - video.currentTime;

		if (Math.abs(driftSeconds) > 15) {
			video.currentTime = targetSeconds;
			video.playbackRate = 1;
		} else if (driftSeconds > 0.75) {
			video.playbackRate = driftSeconds > 5 ? 1.1 : 1.05;
		} else if (driftSeconds < -0.75) {
			video.playbackRate = driftSeconds < -5 ? 0.9 : 0.95;
		} else {
			video.playbackRate = 1;
		}

		if (watchSession.effectiveState === "PLAYING") {
			video.play().catch(() => undefined);
		} else {
			video.pause();
		}
	}, [
		currentMediaId,
		videoRef,
		watchSession.basePositionMs,
		watchSession.baseTimeMs,
		watchSession.effectiveState,
		watchSession.mode,
		watchSession.nodeId,
	]);

	useEffect(() => {
		const video = videoRef.current;
		if (!video || !currentMediaId || !currentFileId || !watchSession.sessionId || !watchSession.playerId) return;

		const sendHeartbeat = () => {
			const sessionState = getWatchSessionState();
			if (!sessionState.sessionId || !sessionState.playerId) return;
			const basePositionMs = Math.max(0, Math.round(video.currentTime * 1000));
			const baseTimeMs = Date.now();
			const isBuffering = playerContext.getState().state.isLoading && !video.paused;
			const recoveryIntent =
				sessionState.intent === "PLAYING"
					? WatchSessionIntent.Playing
					: sessionState.intent === "PAUSED"
						? WatchSessionIntent.Paused
						: video.paused
							? WatchSessionIntent.Paused
							: WatchSessionIntent.Playing;

			void watchSessionHeartbeat({
				variables: {
					input: {
						sessionId: sessionState.sessionId,
						playerId: sessionState.playerId,
						isBuffering,
						basePositionMs,
						baseTimeMs,
						recovery: {
							nodeId: sessionState.nodeId ?? currentMediaId,
							fileId: sessionState.fileId ?? currentFileId,
							intent: recoveryIntent,
							basePositionMs: sessionState.basePositionMs ?? basePositionMs,
							baseTimeMs: sessionState.baseTimeMs ?? baseTimeMs,
						},
					},
				},
			})
				.then((result) => {
					const beacon = result.data?.watchSessionHeartbeat;
					if (beacon) {
						const resolvedBeacon = unmask(WatchSessionBeaconFragment, beacon);
						if (resolvedBeacon.players.some((player) => player.id === sessionState.playerId)) {
							setIsWatchSessionRegistered(true);
						}
						applyWatchSessionBeacon(resolvedBeacon);
					}
				})
				.catch((error) => {
					console.error("failed to send watch session heartbeat", error);
					if (sessionState.mode === "SYNCED") {
						setPlayerWatchSession({ connectionWarning: "Watch session connection lost" });
					}
				});
		};
		heartbeatRef.current = sendHeartbeat;

		sendHeartbeat();
		const interval = window.setInterval(sendHeartbeat, 3_000);
		return () => {
			heartbeatRef.current = null;
			window.clearInterval(interval);
		};
	}, [currentFileId, currentMediaId, videoRef, watchSession.playerId, watchSession.sessionId, watchSessionHeartbeat]);

	useEffect(() => {
		if (watchSession.mode !== "SYNCED" || watchSession.lastContactAt == null) return;
		const video = videoRef.current;
		if (!video) return;

		const interval = window.setInterval(() => {
			const stale = Date.now() - (playerContext.getState().watchSession.lastContactAt ?? 0) >= 12_000;
			if (!stale) return;
			video.pause();
			setPlayerWatchSession({ connectionWarning: "Watch session connection lost" });
		}, 1_000);

		return () => window.clearInterval(interval);
	}, [videoRef, watchSession.lastContactAt, watchSession.mode]);

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
			heartbeatRef.current?.();
		};

		const handleCanPlay = () => {
			setPlayerLoading(false);
			heartbeatRef.current?.();
		};
		const handleWaiting = () => {
			setPlayerLoading(true);
			heartbeatRef.current?.();
		};
		const handlePlaying = () => {
			setPlayerLoading(false);
			heartbeatRef.current?.();
		};
		const handleLoadedData = () => {
			setPlayerLoading(false);
			heartbeatRef.current?.();
		};

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
			heartbeatRef.current?.();
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

	const autoplayEnabled = playerContext.getState().state.autoplay && watchSession.mode !== "SYNCED";
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
