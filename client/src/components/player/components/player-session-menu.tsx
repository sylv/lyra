import { LoaderCircle, UserX2, Users } from "lucide-react";
import { useState, type FC } from "react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "../../ui/dropdown-menu";
import { usePlayerSession } from "../player-session";
import { usePlayerRuntimeStore } from "../player-runtime-store";
import { useShowControlsLock } from "../player-visibility";
import { PlayerButton } from "../ui/player-button";
import { WatchSessionActionKind } from "../../../@generated/gql/graphql";

export const PlayerSessionMenu: FC<{ portalContainer: HTMLElement | null }> = ({ portalContainer }) => {
  const { session, sendAction } = usePlayerSession();
  const isFullscreen = usePlayerRuntimeStore((state) => state.isFullscreen);
  const [open, setOpen] = useState(false);
  const [removingPlayerId, setRemovingPlayerId] = useState<string | null>(null);
  useShowControlsLock(open);

  if (!session.sessionId) return null;
  const shareUrl = `${window.location.origin}/?watchSession=${encodeURIComponent(session.sessionId)}`;

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <PlayerButton
          aria-label="Open watch session menu"
          onClick={(event) => {
            event.stopPropagation();
          }}
        >
          <Users className={isFullscreen ? "size-6" : "size-5"} />
        </PlayerButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        portalContainer={portalContainer}
        className="z-80 w-80 border-zinc-700 bg-black/95 p-3 text-zinc-100 shadow-xl shadow-black/40"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="space-y-3">
          <div className="space-y-1">
            <p className="text-sm text-zinc-300">
              {session.mode === "SYNCED" ? "Synced" : "Advisory"} {session.effectiveState ? `· ${session.effectiveState}` : ""}
            </p>
            {session.connectionWarning ? <p className="text-xs text-orange-300">{session.connectionWarning}</p> : null}
          </div>
          <div className="space-y-1">
            <p className="text-xs font-semibold uppercase text-zinc-500">Invite Link</p>
            <input
              readOnly
              value={shareUrl}
              onFocus={(event) => event.currentTarget.select()}
              className="w-full rounded border border-zinc-800 bg-zinc-950 px-3 py-2 text-xs text-zinc-200 outline-none"
            />
          </div>
          <div className="space-y-1">
            <p className="text-xs font-semibold uppercase text-zinc-500">Players</p>
            <div className="space-y-1">
              {session.players.map((player) => (
                <div
                  key={player.id}
                  className="flex items-center justify-between rounded border border-zinc-800/80 bg-zinc-950/80 px-3 py-2"
                >
                  <div>
                    <p className="text-sm">
                      {player.displayUsername}
                      {player.id === session.playerId ? " (this device)" : ""}
                    </p>
                    <p className="text-xs text-zinc-500">
                      {player.isInactive ? "Inactive" : player.isBuffering ? "Buffering" : "Connected"}
                    </p>
                  </div>
                  {player.canRemove ? (
                    <button
                      type="button"
                      className="rounded p-1 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
                      onClick={() => {
                        setRemovingPlayerId(player.id);
                        void sendAction(WatchSessionActionKind.RemovePlayer, { targetPlayerId: player.id })
                          .catch((error) => {
                            console.error("failed to remove player", error);
                          })
                          .finally(() => setRemovingPlayerId(null));
                      }}
                    >
                      {removingPlayerId === player.id ? <LoaderCircle className="size-4 animate-spin" /> : <UserX2 className="size-4" />}
                    </button>
                  ) : null}
                </div>
              ))}
            </div>
          </div>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
