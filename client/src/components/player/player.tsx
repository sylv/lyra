/** biome-ignore-all lint/a11y/useMediaCaption: hls will add captions when available */
/** biome-ignore-all lint/a11y/useSemanticElements: <explanation> */
/** biome-ignore-all lint/a11y/noStaticElementInteractions: <explanation> */
/** biome-ignore-all lint/a11y/useKeyWithClickEvents: <explanation> */
import * as Slider from "@radix-ui/react-slider";
import Hls from "hls.js";
import {
	ArrowLeft,
	Loader2,
	Maximize,
	Minimize,
	Pause,
	Play,
	Volume,
	Volume1,
	Volume2,
	VolumeX,
	XIcon,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState, type FC, type HTMLAttributes } from "react";
import { useStore } from "zustand/react";
import { cn } from "../../lib/utils";
import {
	playerState,
	setPlayerFullscreen,
	setPlayerLoading,
	setPlayerMedia,
	setPlayerMuted,
	setPlayerVolume,
} from "./player-state";
import { graphql, readFragment } from "gql.tada";

const formatTime = (seconds: number): string => {
	if (!seconds || Number.isNaN(seconds)) return "0:00";
	const mins = Math.floor(seconds / 60);
	const secs = Math.floor(seconds % 60);
	return `${mins}:${secs.toString().padStart(2, "0")}`;
};

export const PlayerFrag = graphql(`
	fragment Player on Media {
		id
		name
		seasonNumber
		episodeNumber
		parent {
			name
		}
		defaultConnection {
			id
		}
	}
`);

export const Player = () => {
	const { currentMedia: currentMediaRef, isFullscreen, volume, isMuted, isLoading } = useStore(playerState);
	const currentMedia = readFragment(PlayerFrag, currentMediaRef);

	const [errorMessage, setErrorMessage] = useState<string | null>(null);
	const [duration, setDuration] = useState<number | null>(null);
	const [currentTime, setCurrentTime] = useState<number | null>(null);
	const [bufferedRanges, setBufferedRanges] = useState<Array<{ start: number; end: number }>>([]);
	const [playing, setPlaying] = useState<boolean>(false);
	const [showControls, setShowControls] = useState<boolean>(true);
	const [hoverTime, setHoverTime] = useState<number | null>(null);

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

	// Sync video volume and muted state with player state
	useEffect(() => {
		if (!videoRef.current) return;
		videoRef.current.volume = volume;
		videoRef.current.muted = isMuted;
	}, [volume, isMuted]);

	useEffect(() => {
		if (!videoRef.current) return;

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

	// eg, "6:33pm"
	const finishTime = useMemo(() => {
		if (!duration || !currentTime) return null;
		const remainingTimeMs = (duration - currentTime) * 1000;
		const finishDate = new Date(Date.now() + remainingTimeMs);
		return finishDate.toLocaleTimeString([], {
			hour: "2-digit",
			minute: "2-digit",
		});
	}, [duration, currentTime]);

	const togglePlayPause = () => {
		if (playing) {
			videoRef.current?.pause();
		} else {
			videoRef.current?.play();
		}
	};

	const toggleMute = () => {
		const newMuted = !isMuted;
		setPlayerMuted(newMuted);
		if (videoRef.current) {
			videoRef.current.muted = newMuted;
		}
	};

	const handleVolumeChange = (newVolume: number) => {
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

	const handleProgressClick = (event: React.MouseEvent<HTMLDivElement>) => {
		event.stopPropagation();
		if (!videoRef.current || !duration) return;

		const rect = event.currentTarget.getBoundingClientRect();
		const clickX = event.clientX - rect.left;
		const newTime = (clickX / rect.width) * duration;
		videoRef.current.currentTime = newTime;
	};

	const handleProgressMouseMove = (event: React.MouseEvent<HTMLDivElement>) => {
		if (!duration) return;

		const rect = event.currentTarget.getBoundingClientRect();
		const hoverX = event.clientX - rect.left;
		const hoverTimeValue = (hoverX / rect.width) * duration;
		setHoverTime(Math.max(0, Math.min(duration, hoverTimeValue)));
	};

	const handleProgressMouseLeave = () => {
		setHoverTime(null);
	};

	const toggleFullscreen = (forceExit: boolean = false) => {
		if (isFullscreen || forceExit) {
			document.exitFullscreen().catch(() => false);
			setPlayerFullscreen(false);
		} else {
			containerRef.current?.requestFullscreen({ navigationUI: "hide" }).catch(() => false);
			setPlayerFullscreen(true);
		}
	};

	const handleContainerClick = () => {
		// on double click, toggle fullscreen. otherwise play/pause
		// its done this way to prevent it pausing on double click
		if (doubleClickTimeoutRef.current != null) {
			clearTimeout(doubleClickTimeoutRef.current);
			doubleClickTimeoutRef.current = null;
			toggleFullscreen();
			showControlsTemporarily();
			return;
		}

		doubleClickTimeoutRef.current = setTimeout(() => {
			togglePlayPause();
			showControlsTemporarily();
			doubleClickTimeoutRef.current = null;
		}, 300);
	};

	useEffect(() => {
		const handleKeyDown = (event: KeyboardEvent) => {
			const video = videoRef.current;
			if (!video) return;

			let triggered = true;
			if (event.key === "ArrowLeft") {
				video.currentTime -= 10;
			} else if (event.key === "ArrowRight") {
				video.currentTime += 30;
			} else if (event.key === "f") {
				toggleFullscreen();
			} else if (event.key === "m") {
				toggleMute();
			} else if (event.key === "c") {
				// todo: enable captions
			} else if (event.key === " ") {
				togglePlayPause();
				event.preventDefault();
			} else if (event.key === "Escape") {
				toggleFullscreen(true);
			} else if (
				event.key === "0" ||
				event.key === "1" ||
				event.key === "2" ||
				event.key === "3" ||
				event.key === "4" ||
				event.key === "5" ||
				event.key === "6" ||
				event.key === "7" ||
				event.key === "8" ||
				event.key === "9"
			) {
				const seekTo = (parseInt(event.key) / 10) * (duration ?? 0);
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
				toggleFullscreen(true);
			}
		};

		document.addEventListener("keydown", handleKeyDown);
		document.addEventListener("fullscreenchange", handleFullscreenChange);
		return () => {
			document.removeEventListener("keydown", handleKeyDown);
			document.removeEventListener("fullscreenchange", handleFullscreenChange);
		};
	}, [duration, toggleFullscreen, togglePlayPause]);

	if (!currentMedia) {
		return null;
	}

	const containerClasses = cn(
		isFullscreen
			? "fixed inset-0 z-50 bg-black"
			: "fixed bottom-4 right-4 rounded-xl overflow-hidden shadow-2xl bg-black w-[32rem] max-w-[80dvw]",
	);

	const progressPercent = duration ? ((currentTime ?? 0) / duration) * 100 : 0;

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

			{/* Loading indicator */}
			{isLoading && (
				<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
					<Loader2 className="w-12 h-12 text-white animate-spin" />
				</div>
			)}

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
									toggleFullscreen(true);
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
								{/* todo */}
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
				<div
					onClick={(event) => event.stopPropagation()}
					className={cn(
						"absolute bottom-0 left-0 right-0 transition-opacity duration-300 group cursor-default !pt-1",
						showControls ? "opacity-100" : "opacity-0",
						isFullscreen ? "p-6" : "p-4",
					)}
				>
					{/* Time indicators */}
					<div className="flex justify-between text-sm text-white/80">
						<span>{formatTime(currentTime || 0)}</span>
						<span>{formatTime(duration || 0)}</span>
					</div>

					{/* Progress bar */}
					<div
						className="py-2 my-2 cursor-pointer"
						onClick={handleProgressClick}
						onMouseMove={handleProgressMouseMove}
						onMouseLeave={handleProgressMouseLeave}
						role="slider"
						tabIndex={0}
						aria-label="Seek video"
						aria-valuemin={0}
						aria-valuemax={duration || 100}
						aria-valuenow={currentTime || 0}
					>
						<div className="relative h-1 bg-white/15 group-hover:h-2 transition-all rounded-md">
							<div className="h-full bg-white/80 transition-all rounded-md" style={{ width: `${progressPercent}%` }} />
							{/* Render all buffered segments */}
							{bufferedRanges.map((range) => {
								if (!duration) return null;
								const startPercent = (range.start / duration) * 100;
								const widthPercent = ((range.end - range.start) / duration) * 100;
								return (
									<div
										key={`${range.start}-${range.end}`}
										className="h-full absolute top-0 bg-white/15 transition-all"
										style={{
											left: `${startPercent}%`,
											width: `${widthPercent}%`,
										}}
									/>
								);
							})}
							{hoverTime && (
								<div
									className="absolute top-0 bottom-0"
									style={{
										left: `${(hoverTime / (duration || 1)) * 100}%`,
									}}
								>
									<div className={cn("absolute -top-1 bottom-0 w-0.5 shadow-lg z-20 bg-white/40")} />
									<div className={cn("absolute -top-8 bg-black/60 px-2 py-0.5 rounded-lg text-sm -translate-x-1/2")}>
										{formatTime(hoverTime)}
									</div>
								</div>
							)}
						</div>
					</div>

					{/* Control buttons */}
					<div className="flex items-center justify-between">
						{/* Left side */}
						<div className="flex items-center gap-2">
							<PlayerButton
								aria-label={playing ? "Pause" : "Play"}
								onClick={(e) => {
									e.stopPropagation();
									togglePlayPause();
								}}
							>
								{playing ? <Pause className="w-6 h-6 text-white" /> : <Play className="w-6 h-6 text-white" />}
							</PlayerButton>
							<VolumeControl
								volume={volume}
								isMuted={isMuted}
								onVolumeChange={handleVolumeChange}
								onToggleMute={toggleMute}
							/>
						</div>
						{/* Right side */}
						<div className="flex items-center gap-4">
							{finishTime && <span className="text-sm">Finishes at {finishTime}</span>}
							<PlayerButton
								aria-label={isFullscreen ? "Exit fullscreen" : "Enter fullscreen"}
								onClick={(e) => {
									e.stopPropagation();
									toggleFullscreen();
								}}
							>
								{isFullscreen ? <Minimize className="w-5 h-5" /> : <Maximize className="w-5 h-5" />}
							</PlayerButton>
						</div>
					</div>
				</div>
			</div>

			{errorMessage && (
				<div className="absolute inset-0 flex items-center justify-center pointer-events-none">
					<div className="text-white text-center p-4 pointer-events-auto">
						<p>{errorMessage}</p>
					</div>
				</div>
			)}
		</div>
	);
};

const VolumeControl: FC<{
	volume: number;
	isMuted: boolean;
	onVolumeChange: (volume: number) => void;
	onToggleMute: () => void;
}> = ({ volume, isMuted, onVolumeChange, onToggleMute }) => {
	const [showSlider, setShowSlider] = useState(false);

	const getVolumeIcon = () => {
		if (isMuted || volume === 0) return <VolumeX className="w-5 h-5" />;
		if (volume < 0.33) return <Volume className="w-5 h-5" />;
		if (volume < 0.66) return <Volume1 className="w-5 h-5" />;
		return <Volume2 className="w-5 h-5" />;
	};

	const handleSliderChange = (value: number[]) => {
		const newVolume = value[0];
		onVolumeChange(newVolume);
	};

	return (
		<div
			className="relative flex items-center"
			onMouseEnter={() => setShowSlider(true)}
			onMouseLeave={() => setShowSlider(false)}
		>
			<PlayerButton
				aria-label={isMuted ? "Unmute" : "Mute"}
				onClick={(e) => {
					e.stopPropagation();
					onToggleMute();
				}}
			>
				{getVolumeIcon()}
			</PlayerButton>

			{/* Volume slider - extended hover area with no gap */}
			<div
				className={cn(
					"absolute left-full flex items-center transition-all duration-200",
					showSlider ? "opacity-100 translate-x-0" : "opacity-0 -translate-x-2 pointer-events-none",
				)}
			>
				<div className="py-6 px-2 flex items-center">
					<Slider.Root
						className="relative flex items-center w-20 h-5 cursor-pointer"
						value={[isMuted ? 0 : volume]}
						max={1}
						step={0.05}
						onValueChange={handleSliderChange}
					>
						<Slider.Track className="bg-white/20 relative grow rounded-full h-1">
							<Slider.Range className="absolute bg-white rounded-full h-full" />
						</Slider.Track>
						<Slider.Thumb className="block w-3 h-3 bg-white rounded-full hover:bg-white/90 focus:outline-none focus:ring-2 focus:ring-white/50" />
					</Slider.Root>
				</div>
			</div>
		</div>
	);
};

const PlayerButton: FC<HTMLAttributes<HTMLButtonElement>> = ({ children, ...props }) => {
	return (
		<button
			type="button"
			className="p-3 hover:bg-zinc-600/30 hover:backdrop-blur-md rounded-lg transition-colors text-white"
			{...props}
		>
			{children}
		</button>
	);
};
