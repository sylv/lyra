import { useState, type FC, type FormEvent } from "react";
import { useMutation } from "urql";
import type { LibraryCardFragment as LibraryCardData } from "../../../@generated/gql/graphql";
import { Button, ButtonStyle } from "../../button";
import { DirectoryPicker } from "../../directory-picker";
import { Input } from "../../input";
import { Modal, ModalBody, ModalHeader } from "../../modal";
import { CreateLibraryMutation, UpdateLibraryMutation } from "./queries";

interface LibraryFormModalProps {
	activeForm:
		| { mode: "create" }
		| {
				mode: "edit";
				library: LibraryCardData;
		  };
	onClose: () => void;
}

export const LibraryFormModal: FC<LibraryFormModalProps> = ({ activeForm, onClose }) => {
	const title = activeForm.mode === "edit" ? "Edit Library" : "New Library";
	const description =
		activeForm.mode === "edit"
			? "Update the display name or scan root for this library."
			: "Pick a name and the root directory Lyra should scan.";

	return (
		<Modal open onOpenChange={(open) => !open && onClose()} className="w-[min(44rem,calc(100vw-2rem))]">
			<ModalHeader height="5.25rem">
				<div className="min-w-0">
					<div>{title}</div>
					<p className="mt-1 text-sm font-normal text-zinc-400">{description}</p>
				</div>
			</ModalHeader>
			<ModalBody>
				<LibraryForm
					mode={activeForm.mode}
					library={activeForm.mode === "edit" ? activeForm.library : undefined}
					onClose={onClose}
				/>
			</ModalBody>
		</Modal>
	);
};

const LibraryForm: FC<{
	mode: "create" | "edit";
	onClose: () => void;
	library?: LibraryCardData;
}> = ({ mode, onClose, library }) => {
	const [{ fetching: creating }, createLibrary] = useMutation(CreateLibraryMutation);
	const [{ fetching: updating }, updateLibrary] = useMutation(UpdateLibraryMutation);
	const [libraryName, setLibraryName] = useState(library?.name ?? "");
	const [selectedPath, setSelectedPath] = useState<string | null>(library?.path ?? null);
	const [error, setError] = useState<string | null>(null);
	const submitting = creating || updating;

	const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
		event.preventDefault();

		if (!libraryName.trim() || !selectedPath) {
			return;
		}

		setError(null);

		try {
			if (mode === "edit" && library) {
				const result = await updateLibrary({
					libraryId: library.id,
					name: libraryName.trim(),
					path: selectedPath,
				});
				if (result.error) {
					throw result.error;
				}
			} else {
				const result = await createLibrary({
					name: libraryName.trim(),
					path: selectedPath,
				});
				if (result.error) {
					throw result.error;
				}
			}

			setLibraryName("");
			setSelectedPath(null);
			onClose();
		} catch (nextError) {
			setError(nextError instanceof Error ? nextError.message : "Failed to save library");
		}
	};

	return (
		<form onSubmit={handleSubmit} className="space-y-5">
			<div className="space-y-2">
				<label className="text-xs font-medium uppercase tracking-wide text-zinc-400" htmlFor={`${mode}-library-name`}>
					Name
				</label>
				<Input
					id={`${mode}-library-name`}
					type="text"
					placeholder="Movies"
					value={libraryName}
					onChange={(event) => setLibraryName(event.target.value)}
					className="w-full"
				/>
			</div>

			<div className="space-y-2">
				<div className="text-xs font-medium uppercase tracking-wide text-zinc-400">Path</div>
				<DirectoryPicker onPathChange={setSelectedPath} initialPath={library?.path ?? "/"} />
			</div>

			{error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}

			<div className="flex justify-end gap-2 pt-4">
				<Button type="button" onClick={onClose} style={ButtonStyle.Transparent} className="px-3">
					Cancel
				</Button>
				<Button
					type="submit"
					disabled={!libraryName.trim() || !selectedPath || submitting}
					loading={submitting}
					style={ButtonStyle.White}
					className="px-4"
				>
					{mode === "edit" ? "Save Changes" : "Create Library"}
				</Button>
			</div>
		</form>
	);
};
