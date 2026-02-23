import { useMutation, useQuery } from "@apollo/client/react";
import { graphql } from "gql.tada";
import { Folder, Plus, X } from "lucide-react";
import { useState, type FC } from "react";
import { DirectoryPicker } from "../../directory-picker";
import { SetupModalStep } from "../setup-modal-step";

interface CreateLibraryStepProps {
	refetch: () => Promise<void>;
}

const GET_LIBRARIES = graphql(`
	query GetLibraries {
		libraries {
			id
			name
			path
		}
	}
`);

const CREATE_LIBRARY = graphql(`
	mutation CreateLibrary($name: String!, $path: String!) {
		createLibrary(name: $name, path: $path) {
			id
			name
			path
		}
	}
`);

export const CreateLibraryStep: FC<CreateLibraryStepProps> = ({ refetch }) => {
	const [showAddForm, setShowAddForm] = useState(false);
	const { data: librariesData } = useQuery(GET_LIBRARIES);
	const libraries = librariesData?.libraries || [];

	return (
		<SetupModalStep loading={false} disabled={libraries.length === 0} onSubmit={() => refetch()} centered={false}>
			{/* Add Library Button/Form */}
			{!showAddForm ? (
				<div className="grid grid-cols-4 gap-4 mb-6">
					{libraries.map((library) => (
						<div
							key={library.id}
							className="aspect-square bg-zinc-800 rounded border border-zinc-700 p-4 flex flex-col items-center justify-center text-center"
						>
							<Folder className="size-8 text-indigo-500 mb-2" />
							<h3 className="font-medium text-sm mb-1 truncate w-full">{library.name}</h3>
							<p className="text-xs text-zinc-400 truncate w-full" title={library.path}>
								{library.path}
							</p>
						</div>
					))}

					<button
						type="button"
						onClick={() => setShowAddForm(true)}
						className="aspect-square bg-zinc-800 rounded border border-zinc-700 border-dashed hover:border-zinc-600 hover:bg-zinc-700 transition-colors flex flex-col items-center justify-center text-zinc-400 hover:text-zinc-300"
					>
						<Plus className="size-8 mb-2" />
						<span className="text-sm">Add Library</span>
					</button>
				</div>
			) : (
				<CreateLibraryForm onClose={() => setShowAddForm(false)} />
			)}
		</SetupModalStep>
	);
};

const CreateLibraryForm: FC<{ onClose: () => void }> = ({ onClose }) => {
	const [createLibrary, { loading: creating }] = useMutation(CREATE_LIBRARY, {
		refetchQueries: [GET_LIBRARIES],
	});

	const [libraryName, setLibraryName] = useState("");
	const [selectedPath, setSelectedPath] = useState<string | null>(null);
	const handleAddLibrary = async () => {
		if (!libraryName.trim() || !selectedPath) return;
		await createLibrary({
			variables: {
				name: libraryName.trim(),
				path: selectedPath,
			},
		});

		// Reset form
		setLibraryName("");
		setSelectedPath(null);
		onClose();
	};

	return (
		<div className="flex flex-col gap-4">
			<input
				type="text"
				placeholder="Library name"
				value={libraryName}
				onChange={(e) => setLibraryName(e.target.value)}
				className="w-full px-3 py-2 text-sm rounded-md bg-zinc-900 border border-zinc-700 outline-none"
			/>

			<DirectoryPicker onPathChange={setSelectedPath} initialPath="/" />

			<div className="flex justify-between gap-2">
				<button type="button" onClick={onClose} className="px-3 py-1 text-xs rounded hover:underline">
					Cancel
				</button>
				<button
					type="button"
					onClick={handleAddLibrary}
					disabled={!libraryName.trim() || !selectedPath || creating}
					className="px-3 py-1 text-xs bg-indigo-600 hover:bg-indigo-700 disabled:bg-zinc-600 disabled:cursor-not-allowed rounded"
				>
					{creating ? "Adding..." : "Add"}
				</button>
			</div>
		</div>
	);
};
