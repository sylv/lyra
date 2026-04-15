import type { FC } from "react";
import type { LibraryCardFragment as LibraryCardData } from "../../../@generated/gql/graphql";
import { Button, ButtonStyle } from "../../button";
import { Modal, ModalBody, ModalFooter, ModalHeader } from "../../modal";

interface ConfirmDeleteLibraryModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  submitting: boolean;
  error: string | null;
  library: LibraryCardData;
}

export const ConfirmDeleteLibraryModal: FC<ConfirmDeleteLibraryModalProps> = ({
  open,
  onOpenChange,
  onConfirm,
  submitting,
  error,
  library,
}) => (
  <Modal open={open} onOpenChange={onOpenChange} className="w-[min(30rem,calc(100vw-2rem))]">
    <ModalHeader>Confirm?</ModalHeader>
    <ModalBody className="space-y-4 px-8 pt-4 pb-8">
      <div className="space-y-2">
        <p className="text-sm text-zinc-300">
          Delete <span className="font-medium text-zinc-100">{library.name}</span>?
        </p>
        <p className="text-sm text-zinc-500">
          This removes the library and its scanned media records from Lyra. Files on disk are not deleted.
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
        Delete Library
      </Button>
    </ModalFooter>
  </Modal>
);
