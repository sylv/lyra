import { CheckCheckIcon, FileWarningIcon, PlayIcon } from "lucide-react";
import { Fragment, type FC, type ReactNode } from "react";
import { useNavigate } from "react-router";
import { cn } from "../lib/utils";
import { playNode } from "./player/store/player-store";
import { UnplayedItemsTab } from "./unplayed-items-tab";

interface PlayWrapperProps {
  itemId?: string | null;
  path: string;
  unavailable?: boolean | null;
  watchProgressHint?: number | null;
  children: ReactNode;
}

export const PlayWrapper: FC<PlayWrapperProps> = ({ children, path, itemId, unavailable, watchProgressHint }) => {
  const navigate = useNavigate();
  const playableItemId = unavailable ? null : itemId;

  return (
    <div className="group/play relative inline-block overflow-hidden rounded-sm">
      {playableItemId && (
        <button
          type="button"
          className={cn(
            "absolute left-0 top-0 z-10 flex h-full w-full cursor-pointer items-center justify-center border-2 border-white/80 opacity-0 rounded-md",
            "transition-opacity duration-75 group-hover/play:opacity-100",
          )}
          onClick={() => {
            playNode(playableItemId, true);
            navigate(path);
          }}
        >
          <PlayIcon className="h-10 w-10 text-white" />
        </button>
      )}
      {watchProgressHint && watchProgressHint !== 1 && (
        <Fragment>
          <div
            className="absolute bottom-0 left-0 z-10 h-1 bg-white/80"
            style={{ width: `${watchProgressHint * 100}%` }}
          />
          <div className="absolute bottom-0 left-0 right-0 z-10 h-1 bg-white/20" />
        </Fragment>
      )}
      {watchProgressHint === 1 && (
        <UnplayedItemsTab>
          <CheckCheckIcon className="size-4" strokeWidth={2.5} />
        </UnplayedItemsTab>
      )}
      {unavailable && (
        <div className="absolute left-0 top-0 flex h-full w-full select-none items-center justify-center gap-2 bg-black/60 p-3">
          <FileWarningIcon className="h-6 w-6 text-orange-500" />
          <p className="text-sm font-semibold text-orange-100">Unavailable</p>
        </div>
      )}
      {children}
    </div>
  );
};
