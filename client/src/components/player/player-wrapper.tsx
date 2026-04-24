import { useEffect, type FC } from "react";
import { usePlayerOptionsStore } from "./player-options-store";
import { hydratePlayerFromSnapshot, usePlayerRuntimeStore } from "./player-runtime-store";
import { Player } from "./player";

export const PlayerWrapper: FC = () => {
  const currentItemId = usePlayerRuntimeStore((state) => state.currentItemId);
  const snapshot = usePlayerOptionsStore((state) => state.snapshot);

  useEffect(() => {
    if (currentItemId) return;
    hydratePlayerFromSnapshot();
  }, [currentItemId, snapshot]);

  if (!currentItemId) return null;
  return <Player itemId={currentItemId} />;
};
