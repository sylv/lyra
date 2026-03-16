/* oxlint-disable jsx_a11y/media-has-caption, jsx_a11y/prefer-tag-over-role */
import { useMutation, useQuery } from "@apollo/client/react";
import { useNavigate } from "@tanstack/react-router";
import Hls from "hls.js";
import { ChevronDown, Loader2, XIcon } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type FC } from "react";
import { useStore } from "zustand/react";
import { graphql } from "../../@generated/gql";
import { getPathForNodeData } from "../../lib/getPathForMedia";
import { cn } from "../../lib/utils";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { PlayerButton } from "./components/player-button";
import { PlayerControls } from "./components/player-controls";
import { SkipIntroButton } from "./components/skip-intro-button";
import {
	clearPlayerMedia,
	playerState,
	setPlayerLoading,
	setPlayerMedia,
	setPlayerMuted,
	setPlayerVolume,
	togglePlayerFullscreen,
} from "./player-state";

const NUMBER_REGEX = /^\d$/;
const LANGUAGE_DISPLAY_NAMES =
	typeof Intl !== "undefined" && typeof Intl.DisplayNames === "function"
		? new Intl.DisplayNames(["en"], { type: "language" })
		: null;

const toLanguageName = (value?: string) => {
	if (!value || LANGUAGE_DISPLAY_NAMES == null) {
		return null;
	}

	const trimmed = value.trim();
	if (!trimmed) {
		return null;
	}

	const variants = [
		trimmed.replaceAll("_", "-"),
		trimmed.toLowerCase().replaceAll("_", "-"),
		trimmed.toLowerCase().split("-")[0],
	];
	for (const variant of variants) {
		if (!variant) {
			continue;
		}
		try {
			const label = LANGUAGE_DISPLAY_NAMES.of(variant);
			if (label) {
				return label;
			}
		} catch {
			// ignore invalid language tags
		}
	}

	return null;
};

const getAudioTrackLabel = (
	track: {
		name?: string;
		lang?: string;
		language?: string;
	},
	id: number,
) => {
	const name = track.name?.trim();
	const trackLanguage = toLanguageName(track.lang) || toLanguageName(track.language);
	if (name) {
		const parsedName = toLanguageName(name);
		if (parsedName) {
			return parsedName;
		}
		if (trackLanguage) {
			return `${name} (${trackLanguage})`;
		}
		return name;
	}

	if (trackLanguage) {
		return trackLanguage;
	}

	return `Track ${id + 1}`;
};

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

const UpdateWatchState = graphql(`
	mutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {
		updateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {
			progressPercent
			updatedAt
		}
	}
`);

const ItemPlaybackQuery = graphql(`
	query ItemPlayback($itemId: String!) {
		node(nodeId: $itemId) {
			id
			libraryId
			kind
			name
			properties {
				seasonNumber
				episodeNumber
				runtimeMinutes
			}
			root {
				name
				libraryId
			}
			watchProgress {
				progressPercent
				completed
				updatedAt
			}
			file {
				id
				segments {
					kind
					startMs
					endMs
				}
				timelinePreview {
					positionMs
					endMs
					sheetIntervalMs
					sheetGapSize
					asset {
						id
						width
						height
					}
				}
			}
			previousPlayable {
				id
			}
			nextPlayable {
				id
			}
		}
	}
`);

export const Player: FC<{ itemId: string; autoplay?: boolean; shouldPromptResume?: boolean }> = ({
	itemId,
	autoplay = false,
	shouldPromptResume = false,
}) => {
	const { isFullscreen, volume, isMuted, isLoading } = useStore(playerState);
	const navigate = useNavigate();

	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [bufferedRanges, setBufferedRanges] = useState<Array<{ start: number; end: number }>>([]);
	const [playing, setPlaying] = useState<boolean>(false);
	const [showControls, setShowControls] = useState<boolean>(true);
	const [duration, setDuration] = useState<number>(0);
	const [currentTime, setCurrentTime] = useState<number>(0);
	const [videoAspectRatio, setVideoAspectRatio] = useState<number>(16 / 9);
	const [audioTrackOptions, setAudioTrackOptions] = useState<Array<{ id: number; label: string }>>([]);
	const [selectedAudioTrackId, setSelectedAudioTrackId] = useState<number | null>(null);
	const [isSettingsMenuOpen, setIsSettingsMenuOpen] = useState<boolean>(false);
	const [resumePromptPosition, setResumePromptPosition] = useState<number | null>(null);

	const videoRef = useRef<HTMLVideoElement>(null);
	const hlsRef = useRef<Hls | null>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const doubleClickTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const resumePromptDecisionRef = useRef<"confirm" | "cancel" | null>(null);
	const pendingResumePositionRef = useRef<number | null>(null);
	const pendingStartLoadRef = useRef<((startPosition: number) => void) | null>(null);
	const isControlsPinned = isSettingsMenuOpen;
	const {
		data,
		previousData,
		loading: isItemLoading,
		error: itemLoadError,
	} = useQuery(ItemPlaybackQuery, {
		variables: {
			itemId,
		},
	});
	// Keep the previous item mounted while loading the next/previous item so browser fullscreen is preserved.
	const currentMedia = data?.node ?? (isItemLoading ? previousData?.node : null) ?? null;
	const introSegment = useMemo(() => {
		const segments = currentMedia?.file?.segments;
		if (!Array.isArray(segments)) {
			return null;
		}

		return (
			segments.find(
				(segment) =>
					segment.kind === "INTRO" &&
					typeof segment.startMs === "number" &&
					typeof segment.endMs === "number" &&
					segment.endMs > segment.startMs,
			) ?? null
		);
	}, [currentMedia?.file?.segments]);
	const introProgressPercent = useMemo(() => {
		if (!introSegment) {
			return 0;
		}

		const introDurationMs = introSegment.endMs - introSegment.startMs;
		if (introDurationMs <= 0) {
			return 0;
		}

		const positionMs = currentTime * 1000;
		return Math.max(0, Math.min(1, (positionMs - introSegment.startMs) / introDurationMs));
	}, [currentTime, introSegment]);
	const isInsideIntroSegment = useMemo(() => {
		if (!introSegment) {
			return false;
		}

		const positionMs = currentTime * 1000;
		return positionMs >= introSegment.startMs && positionMs < introSegment.endMs;
	}, [currentTime, introSegment]);

	useEffect(() => {
		if (!isItemLoading) {
			return;
		}

		setErrorMessage(null);
		setPlayerLoading(true);
	}, [isItemLoading]);

	useEffect(() => {
		if (!itemLoadError) {
			return;
		}

		setErrorMessage("Sorry, this item is unavailable");
		setPlayerLoading(false);
	}, [itemLoadError]);

	useEffect(() => {
		setAudioTrackOptions([]);
		setSelectedAudioTrackId(null);
		setIsSettingsMenuOpen(false);
		setResumePromptPosition(null);
		resumePromptDecisionRef.current = null;
		pendingResumePositionRef.current = null;
		pendingStartLoadRef.current = null;
	}, [currentMedia?.id]);

	const handleResumePromptConfirm = () => {
		const resumePosition = pendingResumePositionRef.current;
		const startLoadAt = pendingStartLoadRef.current;
		resumePromptDecisionRef.current = "confirm";
		pendingResumePositionRef.current = null;
		pendingStartLoadRef.current = null;
		setResumePromptPosition(null);

		if (resumePosition == null || !startLoadAt) {
			return;
		}

		if (videoRef.current) {
			videoRef.current.currentTime = resumePosition;
		}
		startLoadAt(resumePosition);
	};

	const handleResumePromptCancel = () => {
		const startLoadAt = pendingStartLoadRef.current;
		resumePromptDecisionRef.current = "cancel";
		pendingResumePositionRef.current = null;
		pendingStartLoadRef.current = null;
		setResumePromptPosition(null);
		startLoadAt?.(-1);
	};

	useEffect(() => {
		if (!videoRef.current || !currentMedia) return;

		if (hlsRef.current != null) {
			hlsRef.current.destroy();
			hlsRef.current = null;
		}

		if (!currentMedia.file) {
			videoRef.current.pause();
			setErrorMessage("Sorry, this item is unavailable");
			setPlayerLoading(false);
			return;
		}

		if (Hls.isSupported()) {
			setErrorMessage(null);
			setPlayerLoading(true);
			const hlsUrl = `/api/hls/stream/${currentMedia.file.id}/master.m3u8`;
			const watchProgressPercent = currentMedia.watchProgress?.completed
				? null
				: currentMedia.watchProgress?.progressPercent;
			const hasResumableWatchProgress =
				typeof watchProgressPercent === "number" &&
				Number.isFinite(watchProgressPercent) &&
				watchProgressPercent > 0 &&
				watchProgressPercent < 1;
			const safeWatchProgressPercent = hasResumableWatchProgress ? watchProgressPercent : 0;
			const runtimeMinutes = currentMedia.properties.runtimeMinutes;
			const runtimeDurationSeconds =
				typeof runtimeMinutes === "number" && Number.isFinite(runtimeMinutes) && runtimeMinutes > 0
					? runtimeMinutes * 60
					: null;
			const clampResumePosition = (durationSeconds: number) => {
				if (!hasResumableWatchProgress) {
					return null;
				}
				const progress = Math.max(0, Math.min(0.999, safeWatchProgressPercent));
				const maxStart = Math.max(0, durationSeconds - 0.5);
				return Math.max(0, Math.min(progress * durationSeconds, maxStart));
			};
			let hasStartedLoading = false;
			const startLoadAt = (startPosition: number) => {
				if (hasStartedLoading) {
					return;
				}
				hasStartedLoading = true;
				hls.startLoad(Number.isFinite(startPosition) ? startPosition : -1);
			};
			const hls = new Hls({
				autoStartLoad: false,
			});
			hlsRef.current = hls;

			const syncAudioTracks = () => {
				const tracks = hls.audioTracks.map((track, id) => ({
					id,
					label: getAudioTrackLabel(track, id),
				}));
				setAudioTrackOptions(tracks);
				setSelectedAudioTrackId(hls.audioTrack >= 0 ? hls.audioTrack : null);
			};

			hls.on(Hls.Events.ERROR, (event, data) => {
				console.error("HLS error:", event, data);
				if (data.fatal) {
					// setErrorMessage("Failed to load video stream");
					setErrorMessage(`${data.type}: ${data.reason}`);
					setPlayerLoading(false);
				}
			});
			hls.on(Hls.Events.MANIFEST_PARSED, () => {
				syncAudioTracks();
				if (!hasResumableWatchProgress) {
					startLoadAt(-1);
					return;
				}

				const levels = hls.levels;
				const levelDurations = levels
					.map((level) => level.details?.totalduration)
					.filter((value): value is number => typeof value === "number" && Number.isFinite(value) && value > 0);
				const durationSeconds = levelDurations[0] ?? runtimeDurationSeconds;
				const resumePosition = durationSeconds == null ? null : clampResumePosition(durationSeconds);

				if (resumePosition != null) {
					if (shouldPromptResume) {
						resumePromptDecisionRef.current = null;
						pendingResumePositionRef.current = resumePosition;
						pendingStartLoadRef.current = startLoadAt;
						setResumePromptPosition(resumePosition);
						return;
					}

					if (videoRef.current) {
						videoRef.current.currentTime = resumePosition;
					}
					startLoadAt(resumePosition);
					return;
				}

				startLoadAt(-1);
			});
			hls.on(Hls.Events.AUDIO_TRACKS_UPDATED, () => {
				syncAudioTracks();
			});
			hls.on(Hls.Events.AUDIO_TRACK_SWITCHED, (_event, data) => {
				if (typeof data.id === "number") {
					setSelectedAudioTrackId(data.id);
				}
			});

			hls.loadSource(hlsUrl);
			hls.attachMedia(videoRef.current);

			return () => {
				setResumePromptPosition(null);
				resumePromptDecisionRef.current = null;
				pendingResumePositionRef.current = null;
				pendingStartLoadRef.current = null;
				hls.destroy();
				if (hlsRef.current === hls) {
					hlsRef.current = null;
				}
			};
		} else {
			setErrorMessage("Sorry, your browser does not support this video.");
		}
	}, [currentMedia, shouldPromptResume]);

	useEffect(() => {
		if (autoplay) {
			return;
		}

		videoRef.current?.pause();
		setPlaying(false);
	}, [autoplay, currentMedia?.id]);

	useEffect(() => {
		if (!videoRef.current) return;
		const video = videoRef.current;

		const handleLoadedMetadata = () => {
			if (video.videoWidth <= 0 || video.videoHeight <= 0) return;
			setVideoAspectRatio(video.videoWidth / video.videoHeight);
		};

		video.addEventListener("loadedmetadata", handleLoadedMetadata);
		return () => {
			video.removeEventListener("loadedmetadata", handleLoadedMetadata);
		};
	}, [currentMedia?.id]);

	useEffect(() => {
		if (!videoRef.current) return;
		videoRef.current.volume = volume;
		videoRef.current.muted = isMuted;
	}, [volume, isMuted]);

	const [updateWatchProgress] = useMutation(UpdateWatchState);

	// watch state handling
	useEffect(() => {
		if (!videoRef.current || !currentMedia) return;
		const media = currentMedia;
		const video = videoRef.current;

		let lastUpdate = Date.now();
		const onTimeUpdate = () => {
			if (Date.now() - lastUpdate < 10_000) return;
			if (!media.file || video.duration <= 0) return;
			lastUpdate = Date.now();
			updateWatchProgress({
				variables: {
					fileId: media.file.id,
					progressPercent: video.currentTime / video.duration,
				},
			}).catch((err: unknown) => {
				console.error("failed to update watch state", err);
			});
		};

		const onSeek = () => {
			// on seek we don't want to "destroy" the watch state that already exists (eg, if the user seeks forward accidentally
			// persisting that would be bad), so we reset the debounce timer forcing a ~10s delay in update
			lastUpdate = Date.now();
		};

		video.addEventListener("timeupdate", onTimeUpdate);
		video.addEventListener("seeked", onSeek);

		return () => {
			video.removeEventListener("timeupdate", onTimeUpdate);
			video.removeEventListener("seeked", onSeek);
		};
	}, [currentMedia?.id, currentMedia?.file?.id, videoRef]);

	useEffect(() => {
		if (!videoRef.current) return;

		// todo: doing this all the time is wasteful, it would make more sense to handle this per-event
		const updatePlayerData = () => {
			const video = videoRef.current;
			if (!video) return;

			if (video.paused) setPlaying(false);
			else setPlaying(true);

			setCurrentTime(video.currentTime);
			setDuration(video.duration);

			// Sync volume state
			setPlayerVolume(video.volume);
			setPlayerMuted(video.muted);

			// Collect all buffered ranges
			const ranges: Array<{ start: number; end: number }> = [];
			for (let i = 0; i < video.buffered.length; i++) {
				ranges.push({
					start: video.buffered.start(i),
					end: video.buffered.end(i),
				});
			}
			setBufferedRanges(ranges);
		};

		const handleLoadStart = () => setPlayerLoading(true);
		const handleCanPlay = () => setPlayerLoading(false);
		const handleWaiting = () => setPlayerLoading(true);
		const handlePlaying = () => setPlayerLoading(false);
		const handleLoadedData = () => setPlayerLoading(false);

		let lastUpdated = 0;
		const debouncedUpdate = () => {
			if (Date.now() - lastUpdated < 300) return;
			lastUpdated = Date.now();
			updatePlayerData();
		};

		videoRef.current.addEventListener("timeupdate", debouncedUpdate);
		videoRef.current.addEventListener("play", updatePlayerData);
		videoRef.current.addEventListener("pause", updatePlayerData);
		videoRef.current.addEventListener("loadedmetadata", updatePlayerData);
		videoRef.current.addEventListener("volumechange", updatePlayerData);
		videoRef.current.addEventListener("loadstart", handleLoadStart);
		videoRef.current.addEventListener("canplay", handleCanPlay);
		videoRef.current.addEventListener("waiting", handleWaiting);
		videoRef.current.addEventListener("playing", handlePlaying);
		videoRef.current.addEventListener("loadeddata", handleLoadedData);

		return () => {
			const video = videoRef.current;
			if (video) {
				video.removeEventListener("timeupdate", debouncedUpdate);
				video.removeEventListener("play", updatePlayerData);
				video.removeEventListener("pause", updatePlayerData);
				video.removeEventListener("loadedmetadata", updatePlayerData);
				video.removeEventListener("volumechange", updatePlayerData);
				video.removeEventListener("loadstart", handleLoadStart);
				video.removeEventListener("canplay", handleCanPlay);
				video.removeEventListener("loadeddata", handleLoadedData);
				video.removeEventListener("waiting", handleWaiting);
				video.removeEventListener("playing", handlePlaying);
			}
		};
	}, [videoRef, currentMedia, setCurrentTime, setDuration, setBufferedRanges, setPlaying]);

	useEffect(() => {
		if (isFullscreen == null || !containerRef.current) return;
		if (isFullscreen) {
			containerRef.current.requestFullscreen({ navigationUI: "hide" }).catch(() => false);
		} else {
			document.exitFullscreen().catch(() => false);
		}
	}, [isFullscreen]);

	const onTogglePlaying = () => {
		if (playing) {
			videoRef.current?.pause();
		} else {
			videoRef.current?.play();
		}
	};

	const onToggleMute = () => {
		const newMuted = !isMuted;
		setPlayerMuted(newMuted);
		if (videoRef.current) {
			videoRef.current.muted = newMuted;
		}
	};

	const onVolumeChange = (newVolume: number) => {
		setPlayerVolume(newVolume);
		if (videoRef.current) {
			videoRef.current.volume = newVolume;
		}
		// Unmute if volume is increased from 0
		if (newVolume > 0 && isMuted) {
			setPlayerMuted(false);
			if (videoRef.current) {
				videoRef.current.muted = false;
			}
		}
	};

	const showControlsTemporarily = () => {
		setShowControls(true);
		if (controlsTimeoutRef.current) {
			clearTimeout(controlsTimeoutRef.current);
			controlsTimeoutRef.current = null;
		}
		if (isControlsPinned) {
			return;
		}
		controlsTimeoutRef.current = setTimeout(() => {
			setShowControls(false);
		}, 3000);
	};

	useEffect(() => {
		if (!isControlsPinned) {
			return;
		}
		setShowControls(true);
		if (controlsTimeoutRef.current) {
			clearTimeout(controlsTimeoutRef.current);
			controlsTimeoutRef.current = null;
		}
	}, [isControlsPinned]);

	useEffect(() => {
		return () => {
			if (controlsTimeoutRef.current) {
				clearTimeout(controlsTimeoutRef.current);
				controlsTimeoutRef.current = null;
			}
			if (doubleClickTimeoutRef.current) {
				clearTimeout(doubleClickTimeoutRef.current);
				doubleClickTimeoutRef.current = null;
			}
		};
	}, []);

	const handleMouseMove = () => {
		showControlsTemporarily();
	};

	const handleContainerClick = () => {
		// on double click, toggle fullscreen. otherwise play/pause
		// its done this way to prevent it pausing on double click
		if (doubleClickTimeoutRef.current != null) {
			clearTimeout(doubleClickTimeoutRef.current);
			doubleClickTimeoutRef.current = null;
			togglePlayerFullscreen();
			showControlsTemporarily();
			return;
		}

		doubleClickTimeoutRef.current = setTimeout(() => {
			onTogglePlaying();
			showControlsTemporarily();
			doubleClickTimeoutRef.current = null;
		}, 300);
	};

	useEffect(() => {
		const handleKeyDown = (event: KeyboardEvent) => {
			if (isSettingsMenuOpen) {
				return;
			}

			const target = event.target as HTMLElement | null;
			if (target?.closest("[data-slot='dialog-content']") || target?.closest("[data-slot='dropdown-menu-content']")) {
				return;
			}

			const video = videoRef.current;
			if (!video) return;

			const isNumber = NUMBER_REGEX.test(event.key);

			let triggered = true;
			if (event.key === "ArrowLeft") {
				video.currentTime -= 10;
			} else if (event.key === "ArrowRight") {
				video.currentTime += 30;
			} else if (event.key === "f") {
				togglePlayerFullscreen();
			} else if (event.key === "m") {
				onToggleMute();
			} else if (event.key === "c") {
				// todo: enable captions
			} else if (event.key === " ") {
				onTogglePlaying();
				event.preventDefault();
			} else if (event.key === "Escape") {
				togglePlayerFullscreen(false);
			} else if (isNumber) {
				const seekTo = (parseInt(event.key, 10) / 10) * duration;
				if (seekTo) {
					video.currentTime = seekTo;
				}
			} else {
				triggered = false;
			}

			if (triggered) {
				event.preventDefault();
				event.stopPropagation();
			}
		};

		// closing browser fullscreen will set the player to also not be fullscreen
		const handleFullscreenChange = () => {
			if (!document.fullscreenElement) {
				togglePlayerFullscreen(false);
			}
		};

		document.addEventListener("keydown", handleKeyDown);
		document.addEventListener("fullscreenchange", handleFullscreenChange);
		return () => {
			document.removeEventListener("keydown", handleKeyDown);
			document.removeEventListener("fullscreenchange", handleFullscreenChange);
		};
	}, [duration, onToggleMute, onTogglePlaying, isSettingsMenuOpen]);

	const onAudioTrackChange = (trackId: number) => {
		const hls = hlsRef.current;
		if (!hls || Number.isNaN(trackId)) {
			return;
		}

		hls.audioTrack = trackId;
		setSelectedAudioTrackId(trackId);
	};

	const onSeek = (time: number) => {
		if (videoRef.current) {
			videoRef.current.currentTime = time;
		}
	};

	const onSkipIntro = () => {
		if (!videoRef.current || !introSegment) {
			return;
		}

		videoRef.current.currentTime = introSegment.endMs / 1000;
	};

	const onPreviousItem = () => {
		const previousItemId = (currentMedia?.previousPlayable as { id: string } | null)?.id;
		if (!previousItemId) {
			return;
		}

		setPlayerMedia(previousItemId, true);
	};

	const onNextItem = () => {
		const nextItemId = (currentMedia?.nextPlayable as { id: string } | null)?.id;
		if (!nextItemId) {
			return;
		}

		setPlayerMedia(nextItemId, true);
	};

	if (!currentMedia) {
		return null;
	}

	const miniPlayerAspectRatio = Math.max(videoAspectRatio, 16 / 9);
	const detailsPath = currentMedia?.libraryId ? getPathForNodeData(currentMedia) : null;
	const timelinePreviewSheets = Array.isArray(currentMedia.file?.timelinePreview)
		? currentMedia.file.timelinePreview
		: [];

	const containerClasses = cn(
		isFullscreen
			? "z-50 fixed inset-0 bg-black outline-none"
			: "z-50 fixed bottom-4 right-4 rounded shadow-2xl bg-black outline-none",
	);

	return (
		<div
			ref={containerRef}
			className={containerClasses}
			style={
				isFullscreen
					? undefined
					: {
							aspectRatio: miniPlayerAspectRatio,
							width: `min(80dvw, max(32rem, calc(18rem * ${miniPlayerAspectRatio})))`,
						}
			}
			onMouseMove={handleMouseMove}
			onMouseLeave={() => {
				if (!isControlsPinned) {
					setShowControls(false);
				}
			}}
		>
			<video
				ref={videoRef}
				className={cn("block w-full h-full bg-black object-contain outline-none", !isFullscreen && "rounded")}
				autoPlay={autoplay}
				controls={false}
				disablePictureInPicture
			/>

			{/* Overlay controls */}
			<div
				className={cn(
					"absolute inset-0 cursor-pointer select-none outline-none focus:outline-none focus-visible:outline-none focus-visible:ring-0",
					!isFullscreen && "rounded",
				)}
				role="button"
				tabIndex={0}
				onClick={handleContainerClick}
				onKeyDown={(event) => {
					if (event.key !== "Enter" && event.key !== " ") {
						return;
					}

					event.preventDefault();
					handleContainerClick();
				}}
				aria-label="Toggle play/pause"
			>
				{/* Vignette overlay */}
				<div
					className={cn(
						"absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/60 transition-opacity duration-300 pointer-events-none",
						showControls ? "opacity-100" : "opacity-0",
						!isFullscreen && "rounded",
					)}
				/>
				{/* Top section */}
				<div
					className={cn(
						"absolute top-0 left-0 right-0 transition-opacity duration-300 flex justify-between items-center",
						showControls ? "opacity-100" : "opacity-0",
						isFullscreen ? "p-6" : "p-4",
					)}
				>
					<div className="flex items-center gap-3 text-white">
						{isFullscreen && (
							<PlayerButton
								aria-label="Go back"
								onClick={(e) => {
									e.stopPropagation();
									togglePlayerFullscreen(false);
								}}
							>
								<ChevronDown className="size-6" />
							</PlayerButton>
						)}
						{currentMedia.root?.name &&
						currentMedia.properties.seasonNumber &&
						currentMedia.properties.episodeNumber ? (
							<button
								type="button"
								className={cn(
									"text-left rounded-sm transition-colors",
									detailsPath ? "cursor-pointer group" : "cursor-default",
								)}
								onClick={(event) => {
									event.stopPropagation();
									if (detailsPath) {
										togglePlayerFullscreen(false);
										navigate({ to: detailsPath as never });
									}
								}}
							>
								<h2 className="text-xl font-semibold group-hover:underline">
									{currentMedia.root.name}: Season {currentMedia.properties.seasonNumber}
								</h2>
								<p className="text-sm text-gray-300">
									Episode {currentMedia.properties.episodeNumber}: {currentMedia.name}
								</p>
							</button>
						) : (
							<button
								type="button"
								className={cn(
									"text-left rounded-sm transition-colors",
									detailsPath ? "cursor-pointer hover:underline" : "cursor-default",
								)}
								onClick={(event) => {
									event.stopPropagation();
									if (detailsPath) {
										togglePlayerFullscreen(false);
										navigate({ to: detailsPath as never });
									}
								}}
							>
								<h2 className="text-xl font-semibold">{currentMedia.name}</h2>
							</button>
						)}
					</div>
					<div className="flex items-center gap-3 text-white">
						<PlayerButton
							aria-label="Close player"
							onClick={(event) => {
								event.stopPropagation();
								clearPlayerMedia();
							}}
						>
							<XIcon className="size-6" />
						</PlayerButton>
					</div>
				</div>

				{introSegment && isInsideIntroSegment && isFullscreen && (
					<div className={cn("absolute right-0 flex justify-end px-4 pointer-events-none bottom-36")}>
						<div className="pointer-events-auto">
							<SkipIntroButton progressPercent={introProgressPercent} onSkip={onSkipIntro} />
						</div>
					</div>
				)}

				{/* Bottom controls */}
				<PlayerControls
					showControls={showControls}
					isFullscreen={!!isFullscreen}
					currentTime={currentTime}
					duration={duration}
					bufferedRanges={bufferedRanges}
					timelinePreviewSheets={timelinePreviewSheets}
					playing={playing}
					volume={volume}
					isMuted={isMuted}
					onSeek={onSeek}
					onTogglePlaying={onTogglePlaying}
					hasPreviousItem={!!currentMedia.previousPlayable}
					hasNextItem={!!currentMedia.nextPlayable}
					onPreviousItem={onPreviousItem}
					onNextItem={onNextItem}
					onToggleMute={onToggleMute}
					onVolumeChange={onVolumeChange}
					onToggleFullscreen={() => togglePlayerFullscreen()}
					audioTrackOptions={audioTrackOptions}
					selectedAudioTrackId={selectedAudioTrackId}
					onAudioTrackChange={onAudioTrackChange}
					isSettingsMenuOpen={isSettingsMenuOpen}
					onSettingsMenuOpenChange={setIsSettingsMenuOpen}
					dropdownPortalContainer={containerRef.current}
				/>
			</div>

			<Dialog
				open={resumePromptPosition != null}
				onOpenChange={(open) => {
					if (open) {
						return;
					}

					if (resumePromptDecisionRef.current) {
						resumePromptDecisionRef.current = null;
						return;
					}

					handleResumePromptCancel();
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
							onClick={handleResumePromptConfirm}
						>
							Resume from {resumePromptPosition == null ? "0:00" : formatResumeTimestamp(resumePromptPosition)}
						</button>
						<button
							type="button"
							className="w-full px-5 py-4 text-left text-sm font-semibold transition-colors bg-zinc-900/95 hover:bg-zinc-800/95"
							onClick={handleResumePromptCancel}
						>
							Start from the beginning
						</button>
					</div>
				</DialogContent>
			</Dialog>

			{/* Loading indicator */}
			{isLoading && (
				<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
					<Loader2 className="size-12 text-white animate-spin" />
				</div>
			)}

			{errorMessage && (
				<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
					<div className="text-white text-center p-4 mt-24 pointer-events-auto">
						<p>{errorMessage}</p>
					</div>
				</div>
			)}
		</div>
	);
};
