import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { useClient, useMutation } from "urql";
import { playerContext } from "../player-context";
import { ItemPlaybackQuery as ItemPlaybackQueryDoc, DisabledSubtitlesHint, SetPreferredAudio } from "../player-queries";

type CurrentMedia = NonNullable<ItemPlaybackQuery["node"]>;

export const useTrackSelection = (
  currentMedia: CurrentMedia | null,
  itemId: string,
  languageHints: string[],
) => {
  const client = useClient();
  const [, setPreferredAudio] = useMutation(SetPreferredAudio);
  const [, disabledSubtitlesHint] = useMutation(DisabledSubtitlesHint);

  const refreshPlaybackQuery = () =>
    client.query(ItemPlaybackQueryDoc, { itemId, languageHints }, { requestPolicy: "network-only" }).toPromise();

  const onAudioTrackChange = (trackId: number | null) => {
    if (trackId === null || Number.isNaN(trackId)) return;
    playerContext.getState().actions.setAudioTrack(trackId);

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
      const activeSubtitleTrackId = playerContext.getState().state.activeSubtitleTrackId;
      const activeSubtitleRenditionId = playerContext.getState().state.activeSubtitleRenditionId;
      setSubtitleTrack(trackId, { manual: false });
      if (trackId === "" && activeSubtitleTrackId && activeSubtitleRenditionId && currentMedia?.defaultFile?.id) {
        disabledSubtitlesHint({
          input: {
            fileId: currentMedia.defaultFile.id,
            trackId: activeSubtitleTrackId,
            renditionId: activeSubtitleRenditionId,
          },
        })
        .catch((err: unknown) => {
          console.error("failed to send subtitles-disabled hint", err);
        });
      }
      return;
    }

    void setSubtitleTrack(trackId, { manual: true });
  };

  return { onAudioTrackChange, onSubtitleTrackChange };
};
