import { useState, type FC } from "react";
import type { FragmentType } from "../../../@generated/gql";
import { ManagementCreateCard } from "../management-card";
import type { UserCardFragment as UserCardData } from "../../../@generated/gql/graphql";
import { UserCard, UserCardFragment } from "./user-card";
import { UserFormModal } from "./user-form-modal";

interface UserManagerProps {
  users: Array<{ id: string } & FragmentType<typeof UserCardFragment>>;
  libraries: Array<{
    id: string;
    name: string;
    createdAt: number;
  }>;
  viewerId?: string | null;
  error?: string | null;
}

export const UserManager: FC<UserManagerProps> = ({ users, libraries, viewerId, error }) => {
  const [activeForm, setActiveForm] = useState<
    | { mode: "create" }
    | {
        mode: "edit";
        user: UserCardData;
      }
    | null
  >(null);

  return (
    <div className="space-y-4">
      {activeForm ? (
        <UserFormModal
          activeForm={activeForm}
          libraries={libraries}
          viewerId={viewerId ?? null}
          onClose={() => setActiveForm(null)}
        />
      ) : null}

      {error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {users.map((user) => (
          <UserCard
            key={user.id}
            user={user}
            viewerId={viewerId ?? null}
            totalUsers={users.length}
            onEdit={(nextUser) => setActiveForm({ mode: "edit", user: nextUser })}
          />
        ))}

        <ManagementCreateCard
          title="New Account"
          description="Create a new account with an invite link"
          onClick={() => setActiveForm({ mode: "create" })}
        />
      </div>
    </div>
  );
};
