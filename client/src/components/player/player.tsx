/** biome-ignore-all lint/a11y/useMediaCaption: hls will add captions when available */
/** biome-ignore-all lint/a11y/useSemanticElements: <explanation> */
/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
/** biome-ignore-all lint/a11y/useKeyWithClickEvents: <explanation> */
import { readFragment, type FragmentOf } from "gql.tada";
import Hls from "hls.js";
import { ArrowLeft, Loader2, XIcon } from "lucide-react";
import { useEffect, useRef, useState, type FC } from "react";
import { useStore } from "zustand/react";
import { cn } from "../../lib/utils";
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

export const Player: FC<{ media: FragmentOf<typeof PlayerFrag> }> = ({ media: mediaRef }) => {
	const currentMedia = readFragment(PlayerFrag, mediaRef);
	const { isFullscreen, volume, isMuted, isLoading } = useStore(playerState);

	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [bufferedRanges, setBufferedRanges] = useState<Array<{ start: number; end: number }>>([]);
	const [playing, setPlaying] = useState<boolean>(false);
	const [showControls, setShowControls] = useState<boolean>(true);
	const [duration, setDuration] = useState<number>(0);
	const [currentTime, setCurrentTime] = useState<number>(0);

	const videoRef = useRef<HTMLVideoElement>(null);
	const hlsRef = useRef<Hls | null>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const controlsTimeoutRef = useRef<NodeJS.Timeout | null>(null);
	const doubleClickTimeoutRef = useRef<NodeJS.Timeout | null>(null);

	useEffect(() => {
		if (!videoRef.current || !currentMedia) return;

		if (!currentMedia.defaultConnection) {
			setErrorMessage("This file isn't available right now");
			return;
		}

		if (Hls.isSupported()) {
			if (hlsRef.current) {
				hlsRef.current.destroy();
			}

			setErrorMessage(null);
			setPlayerLoading(true);
			const hlsUrl = `/api/hls/stream/${currentMedia.defaultConnection.id}/index.m3u8`;
			hlsRef.current = new Hls();
			hlsRef.current.on(Hls.Events.ERROR, (event, data) => {
				console.error("HLS error:", event, data);
				if (data.fatal) {
					// setErrorMessage("Failed to load video stream");
					setErrorMessage(`${data.type}: ${data.reason}`);
					setPlayerLoading(false);
				}
			});

			hlsRef.current.loadSource(hlsUrl);
			hlsRef.current.attachMedia(videoRef.current);
		} else {
			setErrorMessage("Sorry, your browser does not support this video.");
		}
	}, [currentMedia]);

	useEffect(() => {
		if (!videoRef.current) return;
		videoRef.current.volume = volume;
		videoRef.current.muted = isMuted;
	}, [volume, isMuted]);

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
		const handleLoadedData = () => setPlayerLoading(false);
		const handleWaiting = () => setPlayerLoading(true);
		const handlePlaying = () => setPlayerLoading(false);

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
		videoRef.current.addEventListener("loadeddata", handleLoadedData);
		videoRef.current.addEventListener("waiting", handleWaiting);
		videoRef.current.addEventListener("playing", handlePlaying);

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
		}
		controlsTimeoutRef.current = setTimeout(() => {
			setShowControls(false);
		}, 3000);
	};

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
				const seekTo = (parseInt(event.key) / 10) * duration;
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
	}, [duration, onTogglePlaying]);

	const onSeek = (time: number) => {
		if (videoRef.current) {
			videoRef.current.currentTime = time;
		}
	};

	if (!currentMedia) {
		return null;
	}

	const containerClasses = cn(
		isFullscreen
			? "fixed inset-0 z-50 bg-black"
			: "fixed bottom-4 right-4 rounded-xl overflow-hidden shadow-2xl bg-black w-[32rem] max-w-[80dvw]",
	);

	return (
		<div
			ref={containerRef}
			className={containerClasses}
			onMouseMove={handleMouseMove}
			onMouseLeave={() => setShowControls(false)}
		>
			<video
				ref={videoRef}
				className="w-full h-full object-cover aspect-[16/9]"
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
						{currentMedia.parent && currentMedia.seasonNumber && currentMedia.episodeNumber ? (
							<div>
								<h2 className="text-xl font-semibold">
									{currentMedia.parent.name}: Season {currentMedia.seasonNumber}
								</h2>
								<p className="text-sm text-gray-300">
									Episode {currentMedia.episodeNumber}: {currentMedia.name}
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
				/>
			</div>

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
