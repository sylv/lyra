import { Loader2 } from "lucide-react";
import type { FC } from "react";
import { usePlayerRuntimeStore } from "../player-runtime-store";

export const PlayerLoadingIndicator: FC = () => {
  const buffering = usePlayerRuntimeStore((state) => state.buffering);
  if (!buffering) return null;
  return (
    <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
      <Loader2 className="size-12 animate-spin text-white" />
    </div>
  );
};
