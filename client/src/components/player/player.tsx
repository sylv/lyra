/** biome-ignore-all lint/a11y/useMediaCaption: hls will add captions when available */
import Hls from "hls.js";
import { useEffect, useRef, useState } from "react";
import { useStore } from "zustand/react";
import { playerState } from "./player-state";

export const Player = () => {
	const { currentMedia, isFullscreen } = useStore(playerState);
	const [errorMessage, setErrorMessage] = useState<string | null>(null);

	const videoRef = useRef<HTMLVideoElement>(null);
	const hlsRef = useRef<Hls | null>(null);
	const containerRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		if (!videoRef.current || !currentMedia || !currentMedia.default_connection)
			return;
		if (Hls.isSupported()) {
			if (hlsRef.current) {
				hlsRef.current.destroy();
			}

			setErrorMessage(null);
			const hlsUrl = `/api/hls/stream/${currentMedia.default_connection.id}/index.m3u8`;
			hlsRef.current = new Hls();
			hlsRef.current.on(Hls.Events.ERROR, (event, data) => {
				console.error("HLS error:", event, data);
				if (data.fatal) {
					setErrorMessage("Failed to load video stream");
				}
			});

			hlsRef.current.loadSource(hlsUrl);
			hlsRef.current.attachMedia(videoRef.current);
		} else {
			setErrorMessage("Sorry, your browser does not support this video.");
		}
	}, [currentMedia]);

	if (!currentMedia) {
		return null;
	}

	if (!currentMedia.default_connection) {
		setErrorMessage("File is not available");
	}

	const containerClasses = isFullscreen
		? "fixed inset-0 z-50 bg-black"
		: "fixed bottom-4 right-4 rounded-lg overflow-hidden shadow-lg bg-black w-[25em] max-w-[80dvw]";

	return (
		<div ref={containerRef} className={containerClasses}>
			<video
				ref={videoRef}
				className="w-full h-full object-contain aspect-[16/9]"
				autoPlay
				controls
				disablePictureInPicture
			/>
			{errorMessage && (
				<div className="absolute inset-0 flex items-center justify-center bg-black/80">
					<div className="text-white text-center p-4">
						<p>{errorMessage}</p>
					</div>
				</div>
			)}
		</div>
	);
};
