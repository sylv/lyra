import type { FC } from "react";
import type { FragmentType } from "../../../@generated/gql";
import { SessionCard } from "./session-card";
import { SessionCardFragment } from "./queries";

interface SessionListProps {
  sessions: Array<{ id: string } & FragmentType<typeof SessionCardFragment>>;
}

export const SessionList: FC<SessionListProps> = ({ sessions }) => {
  if (sessions.length === 0) {
    return <div className="text-sm text-zinc-400">No active watch sessions.</div>;
  }

  return (
    <div className="flex flex-wrap gap-3">
      {sessions.map((session) => (
        <SessionCard key={session.id} session={session} />
      ))}
    </div>
  );
};
