import { useApolloClient, useMutation } from "@apollo/client/react";
import { Folder, Pencil, Plus } from "lucide-react";
import { useState, type FC, type FormEvent } from "react";
import { graphql } from "../@generated/gql";
import { cn } from "../lib/utils";
import { DirectoryPicker } from "./directory-picker";
import { Input } from "./input";
import { Spinner } from "./ui/spinner";

export const LibrariesQuery = graphql(`
	query GetLibraries {
		libraries {
			id
			name
			path
			lastScannedAt
		}
	}
`);

const CreateLibraryMutation = graphql(`
	mutation CreateLibrary($name: String!, $path: String!) {
		createLibrary(name: $name, path: $path) {
			id
			name
			path
			lastScannedAt
		}
	}
`);

const UpdateLibraryMutation = graphql(`
	mutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!) {
		updateLibrary(libraryId: $libraryId, name: $name, path: $path) {
			id
			name
			path
			lastScannedAt
		}
	}
`);

type LibrarySummary = {
	id: string;
	name: string;
	path: string;
	lastScannedAt?: number | null;
};

interface LibraryManagerProps {
	libraries: LibrarySummary[];
	loading?: boolean;
	className?: string;
}

export const LibraryManager: FC<LibraryManagerProps> = ({ libraries, loading = false, className }) => {
	const [activeForm, setActiveForm] = useState<
		| { mode: "create" }
		| {
				mode: "edit";
				library: LibrarySummary;
		  }
		| null
	>(null);
	const visibleForm = activeForm ?? (!loading && libraries.length === 0 ? { mode: "create" as const } : null);

	return (
		<div className={cn("space-y-4", className)}>
			{visibleForm?.mode === "edit" && (
				<EditLibraryForm
					key={visibleForm.library.id}
					library={visibleForm.library}
					canCancel={libraries.length > 0}
					onClose={() => setActiveForm(null)}
				/>
			)}
			{visibleForm?.mode === "create" && (
				<CreateLibraryForm canCancel={libraries.length > 0} onClose={() => setActiveForm(null)} />
			)}

			<div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
				{libraries.map((library) => (
					<div
						key={library.id}
						className="flex min-h-36 flex-col justify-between rounded border border-zinc-800 bg-zinc-950/70 p-4"
					>
						<div className="space-y-3">
							<div className="flex items-start justify-between gap-3">
								<div className="flex size-10 items-center justify-center rounded border border-zinc-800 bg-zinc-900">
									<Folder className="size-5 text-indigo-400" />
								</div>
								<button
									type="button"
									onClick={() => setActiveForm({ mode: "edit", library })}
									className="inline-flex items-center gap-1 rounded border border-zinc-800 px-2 py-1 text-xs text-zinc-400 transition-colors hover:border-zinc-600 hover:text-zinc-100"
								>
									<Pencil className="size-3.5" />
									Edit
								</button>
							</div>
							<div>
								<h3 className="truncate font-medium">{library.name}</h3>
								<p className="mt-1 break-all text-sm text-zinc-400">{library.path}</p>
							</div>
						</div>
						<div className="mt-4 text-xs text-zinc-500">
							Last scanned {formatLastScannedAt(library.lastScannedAt ?? null)}
						</div>
					</div>
				))}

				<button
					type="button"
					onClick={() => setActiveForm({ mode: "create" })}
					className="flex min-h-36 flex-col items-center justify-center rounded border border-dashed border-zinc-700 bg-zinc-950/40 p-4 text-zinc-400 transition-colors hover:border-zinc-500 hover:bg-zinc-950 hover:text-zinc-200"
				>
					{loading ? (
						<Spinner />
					) : (
						<>
							<Plus className="mb-3 size-7" />
							<span className="text-sm font-medium">Add library</span>
							<span className="mt-1 text-center text-xs text-zinc-500">
								Create another scan root for movies, shows, or mixed media.
							</span>
						</>
					)}
				</button>
			</div>

			{!loading && libraries.length === 0 && (
				<p className="text-sm text-zinc-500">No libraries yet. Add one to start importing media.</p>
			)}
		</div>
	);
};

const formatLastScannedAt = (lastScannedAt: number | null) => {
	if (!lastScannedAt) {
		return "never";
	}

	return new Date(lastScannedAt * 1000).toLocaleString();
};

const CreateLibraryForm: FC<{ canCancel: boolean; onClose: () => void }> = ({ canCancel, onClose }) => (
	<LibraryForm mode="create" canCancel={canCancel} onClose={onClose} />
);

const EditLibraryForm: FC<{ library: LibrarySummary; canCancel: boolean; onClose: () => void }> = ({
	library,
	canCancel,
	onClose,
}) => <LibraryForm mode="edit" library={library} canCancel={canCancel} onClose={onClose} />;

const LibraryForm: FC<{
	mode: "create" | "edit";
	canCancel: boolean;
	onClose: () => void;
	library?: LibrarySummary;
}> = ({ mode, canCancel, onClose, library }) => {
	const client = useApolloClient();
	const [createLibrary, { loading: creating }] = useMutation(CreateLibraryMutation, {
		refetchQueries: [LibrariesQuery],
		awaitRefetchQueries: true,
	});
	const [updateLibrary, { loading: updating }] = useMutation(UpdateLibraryMutation, {
		refetchQueries: [LibrariesQuery],
		awaitRefetchQueries: true,
	});
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
				await updateLibrary({
					variables: {
						libraryId: library.id,
						name: libraryName.trim(),
						path: selectedPath,
					},
				});
			} else {
				await createLibrary({
					variables: {
						name: libraryName.trim(),
						path: selectedPath,
					},
				});
			}

			await client.refetchQueries({ include: "active" });
			setLibraryName("");
			setSelectedPath(null);
			onClose();
		} catch (nextError) {
			setError(nextError instanceof Error ? nextError.message : "Failed to save library");
		}
	};

	return (
		<form onSubmit={handleSubmit} className="space-y-4 rounded border border-zinc-800 bg-zinc-950/70 p-4">
			<div className="flex items-start justify-between gap-3">
				<div>
					<h3 className="font-medium">{mode === "edit" ? "Edit library" : "Add library"}</h3>
					<p className="mt-1 text-sm text-zinc-400">
						{mode === "edit"
							? "Update the display name or scan root for this library."
							: "Pick a name and the root directory Lyra should scan."}
					</p>
				</div>
				{mode === "edit" && library?.lastScannedAt ? (
					<div className="text-right text-xs text-zinc-500">
						<div>Last scanned</div>
						<div>{formatLastScannedAt(library.lastScannedAt)}</div>
					</div>
				) : null}
			</div>

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

			{error && <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p>}

			<div className="flex justify-end gap-2">
				{canCancel && (
					<button type="button" onClick={onClose} className="px-3 py-1 text-sm text-zinc-400 hover:text-zinc-200">
						Cancel
					</button>
				)}
				<button
					type="submit"
					disabled={!libraryName.trim() || !selectedPath || submitting}
					className="rounded bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-zinc-700 disabled:text-zinc-300"
				>
					{submitting ? "Saving..." : mode === "edit" ? "Save changes" : "Add library"}
				</button>
			</div>
		</form>
	);
};
