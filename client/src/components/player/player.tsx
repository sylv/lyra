/** biome-ignore-all lint/a11y/useMediaCaption: hls will add captions when available */
/** biome-ignore-all lint/a11y/useSemanticElements: <explanation> */
/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
/** biome-ignore-all lint/a11y/useKeyWithClickEvents: <explanation> */
import { useMutation } from "@apollo/client/react";
import { graphql, readFragment, type FragmentOf } from "gql.tada";
import Hls from "hls.js";
import { ArrowLeft, Loader2, XIcon } from "lucide-react";
import { useEffect, useRef, useState, type FC } from "react";
import { useStore } from "zustand/react";
import { cn } from "../../lib/utils";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "../ui/dialog";
import { PlayerButton } from "./components/player-button";
import { PlayerControls } from "./components/player-controls";
import {
	playerState,
	setPlayerLoading,
	setPlayerMedia,
	setPlayerMuted,
	setPlayerVolume,
	togglePlayerFullscreen,
} from "./player-state";
import { PlayerFrag } from "./player-wrapper";

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

const UpdateWatchState = graphql(`
	mutation UpdateWatchState($nodeId: String!, $progressPercent: Float!) {
		updateWatchProgress(nodeId: $nodeId, progressPercent: $progressPercent) {
			progressPercent
			updatedAt
		}
	}
`);

export const Player: FC<{ media: FragmentOf<typeof PlayerFrag> }> = ({ media: mediaRef }) => {
	const currentMedia = readFragment(PlayerFrag, mediaRef);
	const { isFullscreen, volume, isMuted, isLoading } = useStore(playerState);

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
	const [isShortcutsDialogOpen, setIsShortcutsDialogOpen] = useState<boolean>(false);

	const videoRef = useRef<HTMLVideoElement>(null);
	const hlsRef = useRef<Hls | null>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const doubleClickTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const isControlsPinned = isSettingsMenuOpen || isShortcutsDialogOpen;

	useEffect(() => {
		setAudioTrackOptions([]);
		setSelectedAudioTrackId(null);
		setIsSettingsMenuOpen(false);
		setIsShortcutsDialogOpen(false);
	}, [currentMedia?.id]);

	useEffect(() => {
		if (!videoRef.current || !currentMedia) return;

		if (!currentMedia.defaultConnection) {
			setErrorMessage("This file isn't available right now");
			return;
		}

		if (Hls.isSupported()) {
			if (hlsRef.current != null) {
				hlsRef.current.destroy();
				hlsRef.current = null;
			}

			setErrorMessage(null);
			setPlayerLoading(true);
			const hlsUrl = `/api/hls/stream/${currentMedia.defaultConnection.id}/index.m3u8`;
			const hls = new Hls();
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
				hls.destroy();
				if (hlsRef.current === hls) {
					hlsRef.current = null;
				}
			};
		} else {
			setErrorMessage("Sorry, your browser does not support this video.");
		}
	}, [currentMedia]);

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
		if (!videoRef.current) return;
		const video = videoRef.current;

		const onVideoLoad = () => {
			// load the watch state
			// todo: prompt the user to see if they want to resume where they left off
			if (currentMedia?.watchProgress) {
				video.currentTime = currentMedia.watchProgress.progressPercent * video.duration;
			}
		};

		let lastUpdate = Date.now();
		const onTimeUpdate = () => {
			if (Date.now() - lastUpdate < 10_000) return;
			lastUpdate = Date.now();
			updateWatchProgress({
				variables: {
					nodeId: currentMedia.id,
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
		video.addEventListener("loadedmetadata", onVideoLoad);
		video.addEventListener("seeked", onSeek);

		return () => {
			video.removeEventListener("timeupdate", onTimeUpdate);
			video.removeEventListener("loadedmetadata", onVideoLoad);
			video.removeEventListener("seeked", onSeek);
		};
	}, [currentMedia?.id, videoRef]);

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
			if (isShortcutsDialogOpen || isSettingsMenuOpen) {
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
	}, [duration, onToggleMute, onTogglePlaying, isShortcutsDialogOpen, isSettingsMenuOpen]);

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

	if (!currentMedia) {
		return null;
	}

	const miniPlayerAspectRatio = Math.max(videoAspectRatio, 16 / 9);

	const containerClasses = cn(
		isFullscreen
			? "z-50 fixed inset-0 bg-black"
			: "z-50 fixed bottom-4 right-4 rounded-xl overflow-hidden shadow-2xl bg-black",
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
				className="w-full h-full object-contain"
				autoPlay
				controls={false}
				disablePictureInPicture
			/>

			{/* Overlay controls */}
			<div
				className="absolute inset-0 cursor-pointer select-none"
				role="button"
				tabIndex={0}
				onClick={handleContainerClick}
				aria-label="Toggle play/pause"
			>
				{/* Vignette overlay */}
				<div
					className={cn(
						"absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-black/60 transition-opacity duration-300 pointer-events-none",
						showControls ? "opacity-100" : "opacity-0",
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
								<ArrowLeft className="w-6 h-6" />
							</PlayerButton>
						)}
						{currentMedia.parent && currentMedia.properties.seasonNumber && currentMedia.properties.episodeNumber ? (
							<div>
								<h2 className="text-xl font-semibold">
									{currentMedia.parent.name}: Season {currentMedia.properties.seasonNumber}
								</h2>
								<p className="text-sm text-gray-300">
									Episode {currentMedia.properties.episodeNumber}: {currentMedia.name}
								</p>
							</div>
						) : (
							<div>
								<h2 className="text-xl font-semibold">{currentMedia.name}</h2>
							</div>
						)}
					</div>
					<div className="flex items-center gap-3 text-white">
						<PlayerButton
							aria-label="Go back"
							onClick={() => {
								setPlayerMedia(null);
							}}
						>
							<XIcon className="w-6 h-6" />
						</PlayerButton>
					</div>
				</div>

				{/* Bottom controls */}
				<PlayerControls
					showControls={showControls}
					isFullscreen={!!isFullscreen}
					currentTime={currentTime}
					duration={duration}
					bufferedRanges={bufferedRanges}
					playing={playing}
					volume={volume}
					isMuted={isMuted}
					onSeek={onSeek}
					onTogglePlaying={onTogglePlaying}
					onToggleMute={onToggleMute}
					onVolumeChange={onVolumeChange}
					onToggleFullscreen={() => togglePlayerFullscreen()}
					audioTrackOptions={audioTrackOptions}
					selectedAudioTrackId={selectedAudioTrackId}
					onAudioTrackChange={onAudioTrackChange}
					onOpenShortcuts={() => setIsShortcutsDialogOpen(true)}
					isSettingsMenuOpen={isSettingsMenuOpen}
					onSettingsMenuOpenChange={setIsSettingsMenuOpen}
					dropdownPortalContainer={containerRef.current}
				/>
			</div>

			<Dialog open={isShortcutsDialogOpen} onOpenChange={setIsShortcutsDialogOpen}>
				<DialogContent
					portalContainer={containerRef.current}
					className="max-w-md"
					onClick={(event) => {
						event.stopPropagation();
					}}
				>
					<DialogHeader>
						<DialogTitle>Player shortcuts</DialogTitle>
						<DialogDescription>Keyboard controls available in the player.</DialogDescription>
					</DialogHeader>
					<div className="space-y-3 text-sm">
						<div className="flex items-center justify-between gap-3">
							<span>Play / pause</span>
							<kbd className="rounded border px-2 py-0.5 font-mono text-xs">Space</kbd>
						</div>
						<div className="flex items-center justify-between gap-3">
							<span>Skip back</span>
							<kbd className="rounded border px-2 py-0.5 font-mono text-xs">Left Arrow (-10s)</kbd>
						</div>
						<div className="flex items-center justify-between gap-3">
							<span>Skip forward</span>
							<kbd className="rounded border px-2 py-0.5 font-mono text-xs">Right Arrow (+30s)</kbd>
						</div>
						<div className="flex items-center justify-between gap-3">
							<span>Toggle mute</span>
							<kbd className="rounded border px-2 py-0.5 font-mono text-xs">M</kbd>
						</div>
						<div className="flex items-center justify-between gap-3">
							<span>Toggle fullscreen</span>
							<kbd className="rounded border px-2 py-0.5 font-mono text-xs">F</kbd>
						</div>
						<div className="flex items-center justify-between gap-3">
							<span>Exit fullscreen</span>
							<kbd className="rounded border px-2 py-0.5 font-mono text-xs">Esc</kbd>
						</div>
						<div className="flex items-center justify-between gap-3">
							<span>Seek to timeline position</span>
							<kbd className="rounded border px-2 py-0.5 font-mono text-xs">0-9</kbd>
						</div>
					</div>
				</DialogContent>
			</Dialog>

			{/* Loading indicator */}
			{isLoading && (
				<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
					<Loader2 className="w-12 h-12 text-white animate-spin" />
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
