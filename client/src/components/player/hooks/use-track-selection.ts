import { useMutation } from "@apollo/client/react";
import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { playerContext, setPlayerState } from "../player-context";
import { ItemPlaybackQuery as ItemPlaybackQueryDoc, SetPreferredAudio, SetPreferredSubtitle } from "../player-queries";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const useTrackSelection = (currentMedia: CurrentMedia | null, itemId: string) => {
	const [setPreferredAudio] = useMutation(SetPreferredAudio, {
		refetchQueries: [{ query: ItemPlaybackQueryDoc, variables: { itemId } }],
	});
	const [setPreferredSubtitle] = useMutation(SetPreferredSubtitle, {
		refetchQueries: [{ query: ItemPlaybackQueryDoc, variables: { itemId } }],
	});

	const onAudioTrackChange = (trackId: number | null) => {
		const setAudioTrack = playerContext.getState().actions.setAudioTrack;
		if (trackId === null) {
			setAudioTrack(0);
			setPlayerState({ selectedAudioTrackId: 0 });
			setPreferredAudio({ variables: { language: null, disposition: null } }).catch((err: unknown) => {
				console.error("failed to save audio preference", err);
			});
			return;
		}

		if (Number.isNaN(trackId)) return;
		setAudioTrack(trackId);
		setPlayerState({ selectedAudioTrackId: trackId });

		const serverTrack = currentMedia?.file?.tracks?.find(
			(track) => track.trackType === "AUDIO" && track.manifestIndex === trackId,
		);
		if (serverTrack?.language != null) {
			setPreferredAudio({
				variables: { language: serverTrack.language, disposition: serverTrack.disposition ?? null },
			}).catch((err: unknown) => {
				console.error("failed to save audio preference", err);
			});
		}
	};

	const onSubtitleTrackChange = (trackId: number | null) => {
		const { setSubtitleDisplay, setSubtitleTrack } = playerContext.getState().actions;
		if (trackId === null || trackId < 0) {
			setSubtitleDisplay(false);
			setSubtitleTrack(-1);
			setPlayerState({ selectedSubtitleTrackId: trackId === null ? null : -1 });
			setPreferredSubtitle({ variables: { language: null, disposition: null } }).catch((err: unknown) => {
				console.error("failed to save subtitle preference", err);
			});
			return;
		}

		if (Number.isNaN(trackId)) return;
		setSubtitleDisplay(true);
		setSubtitleTrack(trackId);
		setPlayerState({ selectedSubtitleTrackId: trackId });

		const serverTrack = currentMedia?.file?.tracks?.find(
			(track) => track.trackType === "SUBTITLE" && track.manifestIndex === trackId,
		);
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
