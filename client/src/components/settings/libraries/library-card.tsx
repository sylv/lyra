import { useApolloClient, useMutation } from "@apollo/client/react";
import { EllipsisVertical, Pencil, Trash2 } from "lucide-react";
import { useState, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../../../@generated/gql";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "../../ui/dropdown-menu";
import { ManagementCard } from "../management-card";
import { ConfirmDeleteLibraryModal } from "./confirm-delete-library-modal";
import { DeleteLibraryMutation, LibrariesQuery } from "./queries";
import { formatLastScannedAt } from "../../../lib/format-last-scanned-at";
import { generateGradientIcon } from "../../../lib/generate-gradient-icon";
import type { LibraryCardFragment as LibraryCardData } from "../../../@generated/gql/graphql";

interface LibraryCardProps {
	library: FragmentType<typeof LibraryCardFragment>;
	onEdit: (library: LibraryCardData) => void;
}

export const LibraryCardFragment = graphql(`
	fragment LibraryCard on Library {
		id
		name
		path
		createdAt
		lastScannedAt
	}
`);

export const LibraryCard: FC<LibraryCardProps> = ({ library: libraryRaw, onEdit }) => {
	const library = unmask(LibraryCardFragment, libraryRaw);
	const client = useApolloClient();
	const [deleteLibrary, { loading: deleting }] = useMutation(DeleteLibraryMutation, {
		refetchQueries: [LibrariesQuery],
		awaitRefetchQueries: true,
	});
	const [error, setError] = useState<string | null>(null);
	const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false);
	const icon = generateGradientIcon(library.createdAt.toString(), { size: 32 });

	const handleDelete = async () => {
		setError(null);

		try {
			await deleteLibrary({
				variables: {
					libraryId: library.id,
				},
			});
			await client.refetchQueries({ include: "active" });
			setIsDeleteConfirmOpen(false);
		} catch (nextError) {
			setError(nextError instanceof Error ? nextError.message : "Failed to delete library");
		}
	};

	return (
		<>
			<ManagementCard
				icon={<img src={icon} alt="Library Icon" className="size-6 rounded-full" />}
				title={library.name}
				subtitle={library.path}
				subtitleClassName="break-all"
				actions={
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<button
								type="button"
								className="-mt-1 -mr-1 rounded-full p-2 hover:bg-zinc-500/10"
								aria-label={`Open actions for ${library.name}`}
							>
								<EllipsisVertical className="size-4" />
							</button>
						</DropdownMenuTrigger>
						<DropdownMenuContent
							align="end"
							className="border-zinc-800 bg-black/95 text-zinc-100 shadow-xl shadow-black/40"
						>
							<DropdownMenuItem className="py-2" onSelect={() => onEdit(library)}>
								<Pencil className="size-4" />
								Edit Library
							</DropdownMenuItem>
							<DropdownMenuSeparator className="bg-zinc-800" />
							<DropdownMenuItem className="py-2" onSelect={() => setIsDeleteConfirmOpen(true)} variant="destructive">
								<Trash2 className="size-4" />
								Delete Library
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				}
				footer={formatLastScannedAt(library.lastScannedAt ?? null)}
			>
				{error ? <p className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</p> : null}
			</ManagementCard>

			<ConfirmDeleteLibraryModal
				open={isDeleteConfirmOpen}
				onOpenChange={setIsDeleteConfirmOpen}
				onConfirm={() => {
					void handleDelete();
				}}
				submitting={deleting}
				error={error}
				library={library}
			/>
		</>
	);
};
