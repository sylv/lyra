import type { FC } from "react";
import { usePlayerRuntimeStore } from "../player-runtime-store";
import { useShowControlsLock } from "../player-visibility";

export const PlayerErrorOverlay: FC = () => {
  const errorMessage = usePlayerRuntimeStore((state) => state.errorMessage);
  useShowControlsLock(errorMessage != null);
  if (!errorMessage) return null;

  return (
    <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
      <div className="pointer-events-auto mt-24 p-4 text-center text-white">
        <p>{errorMessage}</p>
      </div>
    </div>
  );
};
