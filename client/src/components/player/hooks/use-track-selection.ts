import { useMutation } from "@apollo/client/react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { usePlayerContext } from "../player-context";
import { ItemPlaybackQuery as ItemPlaybackQueryDoc, SetPreferredAudio, SetPreferredSubtitle } from "../player-queries";
import { videoState } from "../video-state";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const useTrackSelection = (currentMedia: CurrentMedia | null, itemId: string) => {
	const { engineRef } = usePlayerContext();
	const [setPreferredAudio] = useMutation(SetPreferredAudio, {
		refetchQueries: [{ query: ItemPlaybackQueryDoc, variables: { itemId } }],
	});
	const [setPreferredSubtitle] = useMutation(SetPreferredSubtitle, {
		refetchQueries: [{ query: ItemPlaybackQueryDoc, variables: { itemId } }],
	});

	const onAudioTrackChange = (trackId: number | null) => {
		const engine = engineRef.current;
		if (!engine) return;

		if (trackId === null) {
			// "Auto" — reset to first track and clear preference
			engine.setAudioTrack(0);
			videoState.setState({ selectedAudioTrackId: 0 });
			setPreferredAudio({ variables: { language: null, disposition: null } }).catch((err: unknown) => {
				console.error("failed to save audio preference", err);
			});
			return;
		}

		if (Number.isNaN(trackId)) return;

		engine.setAudioTrack(trackId);
		videoState.setState({ selectedAudioTrackId: trackId });

		// only persist preference when the track has a parseable language
		const serverTrack = currentMedia?.file?.tracks?.find((t) => t.trackType === "AUDIO" && t.manifestIndex === trackId);
		if (serverTrack?.language != null) {
			setPreferredAudio({
				variables: { language: serverTrack.language, disposition: serverTrack.disposition ?? null },
			}).catch((err: unknown) => {
				console.error("failed to save audio preference", err);
			});
		}
	};

	const onSubtitleTrackChange = (trackId: number | null) => {
		const engine = engineRef.current;
		if (!engine) return;

		if (trackId === null || trackId < 0) {
			// "Auto" / "Off" — disable subtitles and clear preference
			engine.setSubtitleDisplay(false);
			engine.setSubtitleTrack(-1);
			videoState.setState({ selectedSubtitleTrackId: trackId === null ? null : -1 });
			setPreferredSubtitle({ variables: { language: null, disposition: null } }).catch((err: unknown) => {
				console.error("failed to save subtitle preference", err);
			});
			return;
		}

		if (Number.isNaN(trackId)) return;

		engine.setSubtitleDisplay(true);
		engine.setSubtitleTrack(trackId);
		videoState.setState({ selectedSubtitleTrackId: trackId });

		// only persist preference when the track has a parseable language
		const serverTrack = currentMedia?.file?.tracks?.find((t) => t.trackType === "SUBTITLE" && t.manifestIndex === trackId);
		if (serverTrack?.language != null) {
			setPreferredSubtitle({
				variables: { language: serverTrack.language, disposition: serverTrack.disposition ?? null },
			}).catch((err: unknown) => {
				console.error("failed to save subtitle preference", err);
			});
		}
	};

	return { onAudioTrackChange, onSubtitleTrackChange };
};
