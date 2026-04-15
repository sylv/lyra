import type { FC } from "react";
import type { UserCardFragment as UserCardData } from "../../../@generated/gql/graphql";
import { Button, ButtonStyle } from "../../button";
import { Modal, ModalBody, ModalFooter, ModalHeader } from "../../modal";

interface ConfirmDeleteUserModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  submitting: boolean;
  error: string | null;
  user: UserCardData;
}

export const ConfirmDeleteUserModal: FC<ConfirmDeleteUserModalProps> = ({
  open,
  onOpenChange,
  onConfirm,
  submitting,
  error,
  user,
}) => (
  <Modal open={open} onOpenChange={onOpenChange} className="w-[min(30rem,calc(100vw-2rem))]">
    <ModalHeader>Confirm?</ModalHeader>
    <ModalBody className="space-y-4 px-8 pt-4 pb-8">
      <div className="space-y-2">
        <p className="text-sm text-zinc-300">
          Delete <span className="font-medium text-zinc-100">{user.username}</span>?
        </p>
        <p className="text-sm text-zinc-500">
          This removes the account permanently. Sessions and watch history tied to this user will be removed too.
        </p>
      </div>
      {error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}
    </ModalBody>
    <ModalFooter className="pt-4">
      <Button type="button" onClick={() => onOpenChange(false)} style={ButtonStyle.Transparent} className="px-3">
        Cancel
      </Button>
      <Button
        type="button"
        onClick={onConfirm}
        loading={submitting}
        style={ButtonStyle.White}
        className="bg-red-950/60 px-4 text-red-100 not-disabled:hover:bg-red-950"
      >
        Delete User
      </Button>
    </ModalFooter>
  </Modal>
);
