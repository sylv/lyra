import type { FC } from "react";
import { usePlayerContext } from "../player-context";

export const PlayerErrorOverlay: FC = () => {
  const errorMessage = usePlayerContext((ctx) => ctx.state.errorMessage);
  if (!errorMessage) return null;
  return (
    <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
      <div className="pointer-events-auto mt-24 p-4 text-center text-white">
        <p>{errorMessage}</p>
      </div>
    </div>
  );
};
