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
    if (trackId === null || Number.isNaN(trackId)) return;
    playerContext.getState().actions.setAudioTrack(trackId);
    setPlayerState({ selectedAudioTrackId: trackId });

    const serverTrack = currentMedia?.defaultFile?.playbackOptions?.audioTracks?.find(
      (track) => track.streamIndex === trackId,
    );
    if (serverTrack?.language != null) {
      setPreferredAudio({ language: serverTrack.language, disposition: null })
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

    const serverTrack = currentMedia?.defaultFile?.subtitleTracks?.find((track) => track.id === trackId);
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
