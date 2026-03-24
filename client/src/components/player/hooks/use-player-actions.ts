import { setPlayerMuted, setPlayerVolume } from "../player-state";
import { usePlayerContext } from "../player-context";
import { videoState } from "../video-state";
import { playerState } from "../player-state";

export const usePlayerActions = () => {
	const { videoRef } = usePlayerContext();

	const togglePlaying = () => {
		const video = videoRef.current;
		if (!video) return;
		if (video.paused) {
			video.play();
		} else {
			video.pause();
		}
	};

	const seekBy = (deltaSeconds: number) => {
		const video = videoRef.current;
		if (!video) return;
		// prefer the live video.duration over cached state for accuracy
		const dur = Number.isFinite(video.duration) && video.duration > 0 ? video.duration : videoState.getState().duration;
		const nextTime = video.currentTime + deltaSeconds;
		video.currentTime = Math.max(0, dur > 0 ? Math.min(dur, nextTime) : nextTime);
	};

	const onSeek = (time: number) => {
		if (videoRef.current) {
			videoRef.current.currentTime = time;
		}
	};

	const onToggleMute = () => {
		const newMuted = !playerState.getState().isMuted;
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
		// unmute if volume is raised from 0
		if (newVolume > 0 && playerState.getState().isMuted) {
			setPlayerMuted(false);
			if (videoRef.current) {
				videoRef.current.muted = false;
			}
		}
	};

	return { togglePlaying, seekBy, onSeek, onToggleMute, onVolumeChange };
};
