import { useMemo, type FC } from "react";
import { Link } from "react-router";
import { XIcon } from "lucide-react";
import { getPathForNode } from "../../../lib/get-path-for-node";
import { cn } from "../../../lib/utils";
import { PLAYER_GLASS_CLASS } from "../constants";
import { PlayerState, closePlayer, usePlayerStore } from "../store/player-store";

export const PlayerTop: FC = () => {
  const status = usePlayerStore((state) => state.status);
  const data = useMemo(() => {
    if (status.state === PlayerState.Resuming) return status.data;
    if (status.state === PlayerState.Mounted) return status.data;
    return null;
  }, [status]);

  const nodeUrl = useMemo(() => {
    if (!data?.node) return;
    return getPathForNode(data.node);
  }, [data]);

  const { textTop, textBottom } = useMemo(() => {
    if (!data?.node) return { textTop: null, textBottom: null };
    const textTop = data.node.root?.properties.displayName || null;
    if (data.node.properties.seasonNumber && data.node.properties.episodeNumber) {
      const index = `S${data.node.properties.seasonNumber}E${data.node.properties.episodeNumber}`;
      const textBottom = `${index} ${data.node.properties.displayName}`;
      return { textTop, textBottom };
    } else {
      return { textTop };
    }
  }, [data]);

  return (
    <div>
      <div className="p-3 flex justify-between">
        <div>
          {nodeUrl && textTop && (
            <Link to={nodeUrl} className={cn("group block px-3 py-2 rounded-lg", PLAYER_GLASS_CLASS)}>
              <div className="text-lg font-bold group-hover:underline">{textTop}</div>
              {textBottom && <div className="text-sm">{textBottom}</div>}
            </Link>
          )}
        </div>
        <div>
          <button
            aria-label="Close player"
            className={cn("rounded-full p-3 transition-colors hover:bg-zinc-600/30", PLAYER_GLASS_CLASS)}
            onClick={(event) => {
              event.stopPropagation();
              closePlayer();
            }}
          >
            <XIcon className="size-10" />
          </button>
        </div>
      </div>
    </div>
  );
};
