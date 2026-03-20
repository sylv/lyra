import { useMutation } from "@apollo/client/react";
import { useState, type FC, type FormEvent } from "react";
import type { UserCardFragment as UserCardData } from "../../../@generated/gql/graphql";
import { Button, ButtonStyle } from "../../button";
import { Input } from "../../input";
import { Modal, ModalBody, ModalHeader } from "../../modal";
import { CheckboxCard } from "../checkbox-card";
import { CreateUserInviteMutation, UpdateUserMutation, UsersManagementQuery } from "./queries";
import { ADMIN_BIT, permissionOptions } from "../../../lib/user-permissions";

interface UserFormModalProps {
	activeForm:
		| { mode: "create" }
		| {
				mode: "edit";
				user: UserCardData;
		  };
	viewerId: string | null;
	onClose: () => void;
}

export const UserFormModal: FC<UserFormModalProps> = ({ activeForm, viewerId, onClose }) => {
	const title = activeForm.mode === "edit" ? "Edit User" : "New User";
	const description =
		activeForm.mode === "edit"
			? "Update the username and permissions for this account."
			: "Create a pending account, then share the generated invite link.";

	return (
		<Modal open onOpenChange={(open) => !open && onClose()} className="w-[min(44rem,calc(100vw-2rem))]">
			<ModalHeader height="5.25rem">
				<div className="min-w-0">
					<div>{title}</div>
					<p className="mt-1 text-sm font-normal text-zinc-400">{description}</p>
				</div>
			</ModalHeader>
			<ModalBody>
				<UserForm
					mode={activeForm.mode}
					user={activeForm.mode === "edit" ? activeForm.user : undefined}
					viewerId={viewerId}
					onClose={onClose}
				/>
			</ModalBody>
		</Modal>
	);
};

const UserForm: FC<{
	mode: "create" | "edit";
	onClose: () => void;
	user?: UserCardData;
	viewerId: string | null;
}> = ({ mode, onClose, user, viewerId }) => {
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
	const isEditingCurrentUser = mode === "edit" && user?.id === viewerId;

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
		<form onSubmit={handleSubmit} className="space-y-5">
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
				{isEditingCurrentUser ? (
					<p className="text-sm text-zinc-500">Your current account cannot change its own permissions.</p>
				) : null}
				<div className="space-y-3">
					{permissionOptions.map((option) => {
						const checked = (permissions & option.bit) !== 0;
						const disabled = submitting || isEditingCurrentUser || (adminEnabled && option.bit !== ADMIN_BIT);
						const checkboxId = `${mode}-user-permission-${option.bit}`;

						return (
							<CheckboxCard
								key={option.bit}
								id={checkboxId}
								checked={checked}
								disabled={disabled}
								title={option.label}
								description={option.description}
								onCheckedChange={(isEnabled) => {
									setPermissions((current) => (isEnabled ? current | option.bit : current & ~option.bit));
								}}
							/>
						);
					})}
				</div>
			</div>

			{error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}

			<div className="flex justify-end gap-2 pt-4">
				<Button type="button" onClick={onClose} style={ButtonStyle.Transparent} className="px-3">
					Cancel
				</Button>
				<Button
					type="submit"
					disabled={!username.trim() || submitting}
					loading={submitting}
					style={ButtonStyle.White}
					className="px-4"
				>
					{mode === "edit" ? "Save Changes" : "Create Invite"}
				</Button>
			</div>
		</form>
	);
};
