import { EllipsisVertical, KeyRound, Pencil, Trash2 } from "lucide-react";
import { useMemo, useState, type FC } from "react";
import { useMutation } from "urql";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import type { UserCardFragment as UserCardData } from "../../../@generated/gql/graphql";
import { describePermissions } from "../../../lib/describe-permissions";
import { formatLastSeen } from "../../../lib/format-last-seen";
import { ADMIN_BIT, VIEW_ALL_LIBRARIES_BIT } from "../../../lib/user-permissions";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../../ui/dropdown-menu";
import { UserAvatar } from "../../user-avatar";
import { ManagementCard } from "../management-card";
import { ConfirmDeleteUserModal } from "./confirm-delete-user-modal";
import { InviteLinkField } from "./invite-link-field";
import { DeleteUserMutation, ResetUserInviteMutation } from "./queries";

interface UserCardProps {
  user: FragmentType<typeof UserCardFragment>;
  viewerId: string | null;
  totalUsers: number;
  onEdit: (user: UserCardData) => void;
}

export const UserCardFragment = graphql(`
  fragment UserCard on User {
    id
    username
    inviteCode
    permissions
    libraries {
      id
    }
    createdAt
    lastSeenAt
  }
`);

export const UserCard: FC<UserCardProps> = ({ user: userRaw, viewerId, totalUsers, onEdit }) => {
  const user = unmask(UserCardFragment, userRaw);
  const [{ fetching: resetting }, resetUserInvite] = useMutation(ResetUserInviteMutation);
  const [{ fetching: deleting }, deleteUser] = useMutation(DeleteUserMutation);
  const [error, setError] = useState<string | null>(null);
  const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false);
  const isViewer = user.id === viewerId;
  const inviteLink = user.inviteCode
    ? `${window.location.origin}/setup/create-account?inviteCode=${encodeURIComponent(user.inviteCode)}`
    : null;
  const permissionLabels = useMemo(() => describePermissions(user.permissions), [user.permissions]);
  const libraryAccessLabel = useMemo(() => {
    if ((user.permissions & ADMIN_BIT) !== 0) {
      return null;
    }
    if ((user.permissions & VIEW_ALL_LIBRARIES_BIT) !== 0) {
      return "All libraries";
    }
    if (user.libraries.length === 0) {
      return null;
    }
    return user.libraries.length === 1 ? "1 library" : `${user.libraries.length} libraries`;
  }, [user.libraries.length, user.permissions]);
  const resetDisabled = resetting || totalUsers <= 1 || isViewer;
  const deleteDisabled = deleting || totalUsers <= 1 || isViewer;

  const handleReset = async () => {
    setError(null);

    try {
      const result = await resetUserInvite({
        userId: user.id,
      });
      if (result.error) {
        throw result.error;
      }
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to reset account");
    }
  };

  const handleDelete = async () => {
    setError(null);

    try {
      const result = await deleteUser({
        userId: user.id,
      });
      if (result.error) {
        throw result.error;
      }
      setIsDeleteConfirmOpen(false);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to delete account");
    }
  };

  const subtext = useMemo(() => {
    const text = [...permissionLabels];
    if (libraryAccessLabel) text.push(libraryAccessLabel);
    if (user.inviteCode) text.unshift("Pending Invite");
    if (user.id === viewerId) text.unshift("You");
    return text.join(", ");
  }, [libraryAccessLabel, permissionLabels, user.inviteCode, user.id, viewerId]);

  return (
    <>
      <ManagementCard
        icon={<UserAvatar createdAt={user.createdAt} alt="User Icon" className="size-6" size={32} />}
        title={user.username}
        subtitle={subtext}
        subtitleClassName="break-all"
        actions={
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <button
                type="button"
                className="-mt-1 -mr-1 rounded-full p-2 hover:bg-zinc-500/10"
                aria-label={`Open actions for ${user.username}`}
              >
                <EllipsisVertical className="size-4" />
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              className="border-zinc-800 bg-black/95 text-zinc-100 shadow-xl shadow-black/40"
            >
              <DropdownMenuItem className="py-2" onSelect={() => onEdit(user)}>
                <Pencil className="size-4" />
                Edit User
              </DropdownMenuItem>
              <DropdownMenuItem
                className="py-2"
                disabled={resetDisabled}
                onSelect={() => {
                  void handleReset();
                }}
              >
                <KeyRound className="size-4" />
                {resetting ? "Resetting..." : "Reset Invite"}
              </DropdownMenuItem>
              <DropdownMenuSeparator className="bg-zinc-800" />
              <DropdownMenuItem
                className="py-2"
                disabled={deleteDisabled}
                onSelect={() => setIsDeleteConfirmOpen(true)}
                variant="destructive"
              >
                <Trash2 className="size-4" />
                Delete User
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        }
        footer={formatLastSeen(user.lastSeenAt)}
      >
        {inviteLink ? <InviteLinkField inviteLink={inviteLink} /> : null}
        {error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}
      </ManagementCard>

      <ConfirmDeleteUserModal
        open={isDeleteConfirmOpen}
        onOpenChange={setIsDeleteConfirmOpen}
        onConfirm={() => {
          void handleDelete();
        }}
        submitting={deleting}
        error={error}
        user={user}
      />
    </>
  );
};
