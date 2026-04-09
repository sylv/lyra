import { TrackDispositionPreference, type ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { useClient, useMutation } from "urql";
import { playerContext, setPlayerState } from "../player-context";
import { ItemPlaybackQuery as ItemPlaybackQueryDoc, SetPreferredAudio, SetPreferredSubtitle } from "../player-queries";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const useTrackSelection = (currentMedia: CurrentMedia | null, itemId: string) => {
	const client = useClient();
	const [, setPreferredAudio] = useMutation(SetPreferredAudio);
	const [, setPreferredSubtitle] = useMutation(SetPreferredSubtitle);

	const refreshPlaybackQuery = () =>
		client.query(ItemPlaybackQueryDoc, { itemId }, { requestPolicy: "network-only" }).toPromise();

	const onAudioTrackChange = (trackId: number | null) => {
		const setAudioTrack = playerContext.getState().actions.setAudioTrack;
		if (trackId === null) {
			setAudioTrack(0);
			setPlayerState({ selectedAudioTrackId: 0 });
			setPreferredAudio({ language: null, disposition: null })
				.then((result) => {
					if (result.error) {
						throw result.error;
					}
					return refreshPlaybackQuery();
				})
				.catch((err: unknown) => {
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
			setPreferredAudio({ language: serverTrack.language, disposition: serverTrack.disposition ?? null })
				.then((result) => {
					if (result.error) {
						throw result.error;
					}
					return refreshPlaybackQuery();
				})
				.catch((err: unknown) => {
					console.error("failed to save audio preference", err);
				});
		}
	};

	const onSubtitleTrackChange = (trackId: string | null) => {
		const { setSubtitleTrack } = playerContext.getState().actions;
		if (trackId === null || trackId === "") {
			setSubtitleTrack(trackId);
			setPlayerState({ selectedSubtitleTrackId: trackId });
			setPreferredSubtitle({ language: null, disposition: null })
				.then((result) => {
					if (result.error) {
						throw result.error;
					}
					return refreshPlaybackQuery();
				})
				.catch((err: unknown) => {
					console.error("failed to save subtitle preference", err);
				});
			return;
		}

		setSubtitleTrack(trackId);
		setPlayerState({ selectedSubtitleTrackId: trackId });

		const serverTrack = currentMedia?.file?.subtitleTracks?.find((track) => track.id === trackId);
		const disposition = serverTrack?.dispositions?.includes("Commentary")
			? TrackDispositionPreference.Commentary
			: serverTrack?.dispositions?.includes("SDH")
				? TrackDispositionPreference.Sdh
				: serverTrack?.dispositions?.includes("Forced")
					? null
					: TrackDispositionPreference.Normal;
		if (serverTrack?.language != null) {
			setPreferredSubtitle({ language: serverTrack.language, disposition })
				.then((result) => {
					if (result.error) {
						throw result.error;
					}
					return refreshPlaybackQuery();
				})
				.catch((err: unknown) => {
					console.error("failed to save subtitle preference", err);
				});
		}
	};

	return { onAudioTrackChange, onSubtitleTrackChange };
};
