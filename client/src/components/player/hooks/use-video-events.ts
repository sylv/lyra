import { useEffect } from "react";
import { setPlayerLoading, setPlayerMuted, setPlayerVolume } from "../player-state";
import { usePlayerContext } from "../player-context";
import { videoState } from "../video-state";

// attaches all video element event listeners once. writes playback state into videoState and
// loading/volume state into playerState. runs with [] deps since videoRef is a stable ref.
export const useVideoEvents = () => {
	const { videoRef } = usePlayerContext();

	useEffect(() => {
		const video = videoRef.current;
		if (!video) return;

		const updatePlayerData = () => {
			const v = videoRef.current;
			if (!v) return;

			videoState.setState({
				playing: !v.paused,
				currentTime: v.currentTime,
				duration: v.duration,
				// reset ended when playback resumes
				...(!v.paused ? { ended: false } : {}),
			});

			setPlayerVolume(v.volume);
			setPlayerMuted(v.muted);

			const ranges: Array<{ start: number; end: number }> = [];
			for (let i = 0; i < v.buffered.length; i++) {
				ranges.push({ start: v.buffered.start(i), end: v.buffered.end(i) });
			}
			videoState.setState({ bufferedRanges: ranges });
		};

		const handleLoadedMetadata = () => {
			const v = videoRef.current;
			if (!v || v.videoWidth <= 0 || v.videoHeight <= 0) return;
			videoState.setState({ videoAspectRatio: v.videoWidth / v.videoHeight });
			updatePlayerData();
		};

		const handleEnded = () => {
			videoState.setState({ ended: true, playing: false });
		};

		const handleLoadStart = () => {
			setPlayerLoading(true);
			videoState.setState({
				currentTime: 0,
				duration: 0,
				bufferedRanges: [],
				ended: false,
				upNextDismissed: false,
				upNextCountdownCancelled: false,
				isUpNextActive: false,
				isItemCardOpen: false,
			});
		};
		const handleCanPlay = () => setPlayerLoading(false);
		const handleWaiting = () => setPlayerLoading(true);
		const handlePlaying = () => setPlayerLoading(false);
		const handleLoadedData = () => setPlayerLoading(false);

		// throttle timeupdate to avoid excessive renders while scrubbing
		let lastUpdated = 0;
		const debouncedUpdate = () => {
			if (Date.now() - lastUpdated < 300) return;
			lastUpdated = Date.now();
			updatePlayerData();
		};

		video.addEventListener("timeupdate", debouncedUpdate);
		video.addEventListener("play", updatePlayerData);
		video.addEventListener("pause", updatePlayerData);
		video.addEventListener("loadedmetadata", handleLoadedMetadata);
		video.addEventListener("volumechange", updatePlayerData);
		video.addEventListener("loadstart", handleLoadStart);
		video.addEventListener("canplay", handleCanPlay);
		video.addEventListener("waiting", handleWaiting);
		video.addEventListener("playing", handlePlaying);
		video.addEventListener("loadeddata", handleLoadedData);
		video.addEventListener("ended", handleEnded);

		return () => {
			video.removeEventListener("timeupdate", debouncedUpdate);
			video.removeEventListener("play", updatePlayerData);
			video.removeEventListener("pause", updatePlayerData);
			video.removeEventListener("loadedmetadata", handleLoadedMetadata);
			video.removeEventListener("volumechange", updatePlayerData);
			video.removeEventListener("loadstart", handleLoadStart);
			video.removeEventListener("canplay", handleCanPlay);
			video.removeEventListener("waiting", handleWaiting);
			video.removeEventListener("playing", handlePlaying);
			video.removeEventListener("loadeddata", handleLoadedData);
			video.removeEventListener("ended", handleEnded);
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, []);
};
