import { useMutation } from "@apollo/client/react";
import { Copy, KeyRound, Pencil, Plus, Trash2 } from "lucide-react";
import prettyMilliseconds from "pretty-ms";
import { useMemo, useState, type FC, type FormEvent } from "react";
import { graphql } from "../@generated/gql";
import { cn } from "../lib/utils";
import { Input } from "./input";
import { Checkbox } from "./ui/checkbox";
import { Spinner } from "./ui/spinner";

export const UsersManagementQuery = graphql(`
	query UsersManagement {
		viewer {
			id
		}
		users {
			id
			username
			inviteCode
			permissions
			lastSeenAt
		}
	}
`);

const CreateUserInviteMutation = graphql(`
	mutation CreateUserInvite($username: String!, $permissions: Int!) {
		createUserInvite(username: $username, permissions: $permissions) {
			id
			username
			inviteCode
			permissions
			lastSeenAt
		}
	}
`);

const UpdateUserMutation = graphql(`
	mutation UpdateUser($userId: String!, $username: String!, $permissions: Int!) {
		updateUser(userId: $userId, username: $username, permissions: $permissions) {
			id
			username
			inviteCode
			permissions
			lastSeenAt
		}
	}
`);

const ResetUserInviteMutation = graphql(`
	mutation ResetUserInvite($userId: String!) {
		resetUserInvite(userId: $userId) {
			id
			username
			inviteCode
			permissions
			lastSeenAt
		}
	}
`);

const DeleteUserMutation = graphql(`
	mutation DeleteUser($userId: String!) {
		deleteUser(userId: $userId)
	}
`);

const ADMIN_BIT = 1 << 0;

const permissionOptions = [
	{
		bit: ADMIN_BIT,
		label: "Admin",
		description: "Full access across Lyra, including all user management actions.",
	},
	{
		bit: 1 << 1,
		label: "Create invites",
		description: "Can issue invite links for pending accounts.",
	},
	{
		bit: 1 << 2,
		label: "Manage users",
		description: "Can create, edit, reset, and delete other accounts.",
	},
	{
		bit: 1 << 3,
		label: "Edit watch state",
		description: "Can update watch progress for other users.",
	},
	{
		bit: 1 << 4,
		label: "View all libraries",
		description: "Can see libraries regardless of narrower assignment rules later on.",
	},
] as const;

type UserRecord = {
	id: string;
	username: string;
	inviteCode?: string | null;
	permissions: number;
	lastSeenAt?: number | null;
};

interface UserManagerProps {
	users: UserRecord[];
	viewerId?: string | null;
	loading?: boolean;
	error?: string | null;
}

export const UserManager: FC<UserManagerProps> = ({ users, viewerId, loading = false, error }) => {
	const [activeForm, setActiveForm] = useState<
		| { mode: "create" }
		| {
				mode: "edit";
				user: UserRecord;
		  }
		| null
	>(null);

	return (
		<div className="space-y-4">
			{activeForm?.mode === "edit" ? <EditUserForm user={activeForm.user} onClose={() => setActiveForm(null)} /> : null}
			{activeForm?.mode === "create" ? <CreateUserForm onClose={() => setActiveForm(null)} /> : null}

			<div className="flex items-start justify-between gap-3">
				<div>
					<h3>Accounts</h3>
					<p className="mt-1 text-sm text-zinc-400">
						Create invite links, edit permissions, and reset accounts back to a pending invite.
					</p>
				</div>
				<button
					type="button"
					onClick={() => setActiveForm({ mode: "create" })}
					className="inline-flex items-center gap-2 rounded border border-zinc-800 bg-zinc-950/70 px-3 py-2 text-sm text-zinc-100 transition-colors hover:border-zinc-600"
				>
					<Plus className="size-4" />
					New user
				</button>
			</div>

			{error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}

			<div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
				{users.map((user) => (
					<UserCard
						key={user.id}
						user={user}
						viewerId={viewerId ?? null}
						totalUsers={users.length}
						onEdit={() => setActiveForm({ mode: "edit", user })}
					/>
				))}

				{loading ? (
					<div className="flex min-h-44 items-center justify-center rounded border border-dashed border-zinc-700 bg-zinc-950/40 p-4">
						<Spinner />
					</div>
				) : null}
			</div>
		</div>
	);
};

const UserCard: FC<{
	user: UserRecord;
	viewerId: string | null;
	totalUsers: number;
	onEdit: () => void;
}> = ({ user, viewerId, totalUsers, onEdit }) => {
	const [resetUserInvite, { loading: resetting }] = useMutation(ResetUserInviteMutation, {
		refetchQueries: [UsersManagementQuery],
		awaitRefetchQueries: true,
	});
	const [deleteUser, { loading: deleting }] = useMutation(DeleteUserMutation, {
		refetchQueries: [UsersManagementQuery],
		awaitRefetchQueries: true,
	});
	const [error, setError] = useState<string | null>(null);
	const isPending = Boolean(user.inviteCode);
	const isViewer = user.id === viewerId;
	const inviteLink = user.inviteCode
		? `${window.location.origin}/setup/create-account?inviteCode=${encodeURIComponent(user.inviteCode)}`
		: null;
	const permissionLabels = useMemo(() => describePermissions(user.permissions), [user.permissions]);
	const resetDisabled = resetting || totalUsers <= 1 || isViewer;
	const deleteDisabled = deleting || totalUsers <= 1 || isViewer;
	const resetTitle =
		totalUsers <= 1
			? "You cannot reset the last remaining account."
			: isViewer
				? "You cannot reset the account you are signed in with."
				: undefined;
	const deleteTitle =
		totalUsers <= 1
			? "You cannot delete the last remaining account."
			: isViewer
				? "You cannot delete the account you are signed in with."
				: undefined;

	const handleReset = async () => {
		setError(null);

		try {
			await resetUserInvite({
				variables: {
					userId: user.id,
				},
			});
		} catch (nextError) {
			setError(nextError instanceof Error ? nextError.message : "Failed to reset account");
		}
	};

	const handleDelete = async () => {
		setError(null);

		try {
			await deleteUser({
				variables: {
					userId: user.id,
				},
			});
		} catch (nextError) {
			setError(nextError instanceof Error ? nextError.message : "Failed to delete account");
		}
	};

	return (
		<div className="flex min-h-56 flex-col justify-between rounded border border-zinc-800 bg-zinc-950/70 p-4">
			<div className="space-y-4">
				<div className="flex items-start justify-between gap-3">
					<div>
						<div className="flex items-center gap-2">
							<h3 className="font-medium">{user.username}</h3>
							{isViewer ? (
								<span className="rounded-full border border-emerald-600/40 bg-emerald-500/10 px-2 py-0.5 text-[11px] text-emerald-300">
									You
								</span>
							) : null}
							<span
								className={cn(
									"rounded-full px-2 py-0.5 text-[11px]",
									isPending
										? "border border-amber-600/40 bg-amber-500/10 text-amber-200"
										: "border border-zinc-700 bg-zinc-900 text-zinc-300",
								)}
							>
								{isPending ? "Pending invite" : "Active"}
							</span>
						</div>
						<p className="mt-1 text-xs text-zinc-500">{formatLastSeen(user.lastSeenAt)}</p>
					</div>
					<button
						type="button"
						onClick={onEdit}
						className="inline-flex items-center gap-1 rounded border border-zinc-800 px-2 py-1 text-xs text-zinc-400 transition-colors hover:border-zinc-600 hover:text-zinc-100"
					>
						<Pencil className="size-3.5" />
						Edit
					</button>
				</div>

				<div className="flex flex-wrap gap-2">
					{permissionLabels.map((label) => (
						<span
							key={label}
							className="rounded-full border border-zinc-700 bg-zinc-900 px-2 py-1 text-[11px] text-zinc-300"
						>
							{label}
						</span>
					))}
				</div>

				{inviteLink ? <InviteLinkField inviteLink={inviteLink} /> : null}

				{error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}
			</div>

			<div className="mt-4 flex flex-wrap gap-2">
				<button
					type="button"
					onClick={handleReset}
					disabled={resetDisabled}
					title={resetTitle}
					className="inline-flex items-center gap-2 rounded border border-zinc-800 px-3 py-2 text-sm text-zinc-200 transition-colors hover:border-zinc-600 disabled:cursor-not-allowed disabled:opacity-60"
				>
					<KeyRound className="size-4" />
					{resetting ? "Resetting..." : "Reset"}
				</button>
				<button
					type="button"
					onClick={handleDelete}
					disabled={deleteDisabled}
					title={deleteTitle}
					className="inline-flex items-center gap-2 rounded border border-red-900/70 px-3 py-2 text-sm text-red-200 transition-colors hover:border-red-700 disabled:cursor-not-allowed disabled:opacity-40"
				>
					<Trash2 className="size-4" />
					Delete
				</button>
			</div>
		</div>
	);
};

const InviteLinkField: FC<{ inviteLink: string }> = ({ inviteLink }) => {
	const [copied, setCopied] = useState(false);

	const handleCopy = async () => {
		try {
			await navigator.clipboard.writeText(inviteLink);
			setCopied(true);
			window.setTimeout(() => setCopied(false), 1500);
		} catch {
			setCopied(false);
		}
	};

	return (
		<div className="space-y-2">
			<div className="text-xs font-medium uppercase tracking-wide text-zinc-500">Invite link</div>
			<div className="flex items-center gap-2">
				<Input value={inviteLink} readOnly className="w-full" />
				<button
					type="button"
					onClick={handleCopy}
					className="inline-flex items-center gap-2 rounded border border-zinc-800 px-3 py-2 text-sm text-zinc-200 transition-colors hover:border-zinc-600"
				>
					<Copy className="size-4" />
					{copied ? "Copied" : "Copy"}
				</button>
			</div>
		</div>
	);
};

const CreateUserForm: FC<{ onClose: () => void }> = ({ onClose }) => <UserForm mode="create" onClose={onClose} />;

const EditUserForm: FC<{ user: UserRecord; onClose: () => void }> = ({ user, onClose }) => (
	<UserForm mode="edit" user={user} onClose={onClose} />
);

const UserForm: FC<{
	mode: "create" | "edit";
	onClose: () => void;
	user?: UserRecord;
}> = ({ mode, onClose, user }) => {
	const [createUserInvite, { loading: creating }] = useMutation(CreateUserInviteMutation, {
		refetchQueries: [UsersManagementQuery],
		awaitRefetchQueries: true,
	});
	const [updateUser, { loading: updating }] = useMutation(UpdateUserMutation, {
		refetchQueries: [UsersManagementQuery],
		awaitRefetchQueries: true,
	});
	const [username, setUsername] = useState(user?.username ?? "");
	const [permissions, setPermissions] = useState(user?.permissions ?? 0);
	const [error, setError] = useState<string | null>(null);
	const adminEnabled = (permissions & ADMIN_BIT) !== 0;
	const submitting = creating || updating;

	const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
		event.preventDefault();
		setError(null);

		try {
			if (mode === "edit" && user) {
				await updateUser({
					variables: {
						userId: user.id,
						username: username.trim(),
						permissions,
					},
				});
			} else {
				await createUserInvite({
					variables: {
						username: username.trim(),
						permissions,
					},
				});
			}

			onClose();
		} catch (nextError) {
			setError(nextError instanceof Error ? nextError.message : "Failed to save user");
		}
	};

	return (
		<form onSubmit={handleSubmit} className="space-y-4 rounded border border-zinc-800 bg-zinc-950/70 p-4">
			<div className="flex items-start justify-between gap-3">
				<div>
					<h3 className="font-medium">{mode === "edit" ? "Edit user" : "Create user"}</h3>
					<p className="mt-1 text-sm text-zinc-400">
						{mode === "edit"
							? "Update the username and permissions for this account."
							: "Create a pending account, then share the generated invite link."}
					</p>
				</div>
				<button type="button" onClick={onClose} className="px-3 py-1 text-sm text-zinc-400 hover:text-zinc-200">
					Cancel
				</button>
			</div>

			<div className="space-y-2">
				<label className="text-xs font-medium uppercase tracking-wide text-zinc-400" htmlFor={`${mode}-user-name`}>
					Username
				</label>
				<Input
					id={`${mode}-user-name`}
					type="text"
					placeholder="alex"
					value={username}
					onChange={(event) => setUsername(event.target.value)}
					className="w-full"
				/>
			</div>

			<div className="space-y-3">
				<div className="text-xs font-medium uppercase tracking-wide text-zinc-400">Permissions</div>
				<div className="space-y-3">
					{permissionOptions.map((option) => {
						const checked = (permissions & option.bit) !== 0;
						const disabled = submitting || (adminEnabled && option.bit !== ADMIN_BIT);

						return (
							<label key={option.bit} className="flex items-start gap-3 rounded border border-zinc-800 px-3 py-3">
								<Checkbox
									checked={checked}
									disabled={disabled}
									onCheckedChange={(nextChecked) => {
										const isEnabled = nextChecked === true;
										setPermissions((current) => (isEnabled ? current | option.bit : current & ~option.bit));
									}}
									className="mt-0.5"
								/>
								<div>
									<div className="text-sm font-medium text-zinc-100">{option.label}</div>
									<p className="mt-1 text-sm text-zinc-400">{option.description}</p>
								</div>
							</label>
						);
					})}
				</div>
			</div>

			{error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}

			<div className="flex justify-end">
				<button
					type="submit"
					disabled={submitting}
					className="inline-flex items-center gap-2 rounded border border-zinc-700 bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950 transition-colors hover:bg-white disabled:cursor-not-allowed disabled:opacity-60"
				>
					{submitting ? <Spinner /> : null}
					{mode === "edit" ? "Save changes" : "Create invite"}
				</button>
			</div>
		</form>
	);
};

const describePermissions = (permissions: number) => {
	if ((permissions & ADMIN_BIT) !== 0) {
		return ["Admin"];
	}

	const labels = permissionOptions
		.filter((option) => option.bit !== ADMIN_BIT && (permissions & option.bit) !== 0)
		.map((option) => option.label);

	return labels.length > 0 ? labels : ["No extra permissions"];
};

const formatLastSeen = (lastSeenAt?: number | null) => {
	if (!lastSeenAt) {
		return "Not signed in yet";
	}

	const seenAt = new Date(lastSeenAt * 1000);
	const now = new Date();

	if (
		seenAt.getFullYear() === now.getFullYear() &&
		seenAt.getMonth() === now.getMonth() &&
		seenAt.getDate() === now.getDate()
	) {
		return "Last seen today";
	}

	const elapsedMs = Math.max(0, now.getTime() - seenAt.getTime());
	return `Last seen ${prettyMilliseconds(elapsedMs, { unitCount: 1, verbose: true })} ago`;
};
