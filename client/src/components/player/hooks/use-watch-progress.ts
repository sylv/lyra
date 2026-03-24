import { useMutation } from "@apollo/client/react";
import { useEffect } from "react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { usePlayerContext } from "../player-context";
import { UpdateWatchState } from "../player-queries";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const useWatchProgress = (currentMedia: CurrentMedia | null) => {
	const { videoRef } = usePlayerContext();
	const [updateWatchProgress] = useMutation(UpdateWatchState);

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

		// on seek we don't want to "destroy" the watch state that already exists (eg, if the user seeks
		// forward accidentally, persisting that would be bad), so we reset the debounce timer.
		const onSeek = () => {
			lastUpdate = Date.now();
		};

		video.addEventListener("timeupdate", onTimeUpdate);
		video.addEventListener("seeked", onSeek);
		return () => {
			video.removeEventListener("timeupdate", onTimeUpdate);
			video.removeEventListener("seeked", onSeek);
		};
	}, [currentMedia?.id, currentMedia?.file?.id, videoRef, updateWatchProgress]);
};
