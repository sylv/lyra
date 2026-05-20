import type { FC } from "react";
import { PlayerPickDialog } from "./components/player-pick-dialog";
import prettyMilliseconds from "pretty-ms";
import { PlayerState, setPlayerStatus, usePlayerStore } from "./store/player-store";

export const PlayerResumeDialog: FC = () => {
  const status = usePlayerStore((state) => state.status);
  if (status.state !== PlayerState.Resuming) return null;
  return (
    <PlayerPickDialog
      options={[
        {
          id: "resume",
          label: `Resume from ${prettyMilliseconds(status.fromTimeMs, { secondsDecimalDigits: 0 })}`,
        },
        {
          id: "restart",
          label: "Play from the beginning",
        },
      ]}
      onSelect={(id) => {
        if (id === "resume") {
          usePlayerStore.setState((state) => {
            state.currentTime = status.fromTimeMs / 1000;
          });
        }

        setPlayerStatus({
          state: PlayerState.Mounted,
          audioTrack: status.audioTrack,
          audioTracks: status.audioTracks,
          subtitleTracks: status.subtitleTracks,
          videoTrack: status.videoTrack,
          videoTracks: status.videoTracks,
          data: status.data,
        });
      }}
    />
  );
};
