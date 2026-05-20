import type { FC } from "react";
import { Spinner } from "../../ui/spinner";
import { usePlayerStore } from "../store/player-store";
import { PlayerUpNext } from "../components/player-up-next";
import { PlayerSkipIntro } from "../components/player-skip-intro";

export const PlayerMiddle: FC = () => {
  const buffering = usePlayerStore((state) => state.buffering);

  return (
    <div className="relative px-3 grow">
      <PlayerSkipIntro />
      <PlayerUpNext />
      {buffering && (
        <div className="absolute inset-0 flex items-center justify-center">
          <Spinner className="size-16" />
        </div>
      )}
    </div>
  );
};
