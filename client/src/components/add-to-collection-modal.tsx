import { FolderPlusIcon, PlusIcon } from "lucide-react";
import { useMemo, useState, type FC, type FormEvent } from "react";
import { useMutation, useQuery } from "urql";
import { graphql } from "../@generated/gql";
import { CollectionResolverKind, CollectionVisibility } from "../@generated/gql/graphql";
import { Button, ButtonStyle } from "./button";
import { Input } from "./input";
import { Modal, ModalBody, ModalFooter, ModalHeader } from "./modal";

const EditableCollectionsQuery = graphql(`
	query EditableCollections {
		collections {
			id
			name
			canEdit
			resolverKind
		}
	}
`);

const CreateCollectionMutation = graphql(`
	mutation CreatePrivateCollection(
		$name: String!
		$resolverKind: CollectionResolverKind!
		$visibility: CollectionVisibility!
	) {
		createCollection(name: $name, resolverKind: $resolverKind, visibility: $visibility) {
			id
			name
		}
	}
`);

const AddNodeToCollectionMutation = graphql(`
	mutation AddNodeToCollection($collectionId: String!, $nodeId: String!) {
		addNodeToCollection(collectionId: $collectionId, nodeId: $nodeId) {
			id
			name
		}
	}
`);

export const AddToCollectionModal: FC<{
	nodeId: string;
	open: boolean;
	onOpenChange: (open: boolean) => void;
}> = ({ nodeId, open, onOpenChange }) => {
	const [{ data, fetching }] = useQuery({
		query: EditableCollectionsQuery,
		pause: !open,
	});
	const [{ fetching: creating }, createCollection] = useMutation(CreateCollectionMutation);
	const [{ fetching: adding }, addNodeToCollection] = useMutation(AddNodeToCollectionMutation);
	const [newCollectionName, setNewCollectionName] = useState("");
	const [error, setError] = useState<string | null>(null);

	const manualCollections = useMemo(() => {
		if (!open) return [];
		if (!data?.collections) return [];
		return data.collections.filter(
			(collection) => collection.canEdit && collection.resolverKind === CollectionResolverKind.Manual,
		);
	}, [open, data?.collections]);

	const handleAdd = async (collectionId: string) => {
		setError(null);
		const result = await addNodeToCollection({ collectionId, nodeId });
		if (result.error) {
			setError(result.error.message);
			return;
		}
		onOpenChange(false);
	};

	const handleCreate = async (event: FormEvent<HTMLFormElement>) => {
		event.preventDefault();
		if (!newCollectionName.trim()) return;
		setError(null);

		const created = await createCollection({
			name: newCollectionName.trim(),
			resolverKind: CollectionResolverKind.Manual,
			visibility: CollectionVisibility.Private,
		});
		if (created.error || !created.data?.createCollection) {
			setError(created.error?.message ?? "Failed to create collection");
			return;
		}

		const added = await addNodeToCollection({
			collectionId: created.data.createCollection.id,
			nodeId,
		});
		if (added.error) {
			setError(added.error.message);
			return;
		}

		setNewCollectionName("");
		onOpenChange(false);
	};

	return (
		<Modal open={open} onOpenChange={onOpenChange} className="w-[min(34rem,calc(100vw-2rem))]">
			<ModalHeader>Add to Collection</ModalHeader>
			<ModalBody className="space-y-6">
				<div className="space-y-2">
					<div className="text-xs font-medium uppercase tracking-wide text-zinc-400">Your Collections</div>
					<div className="space-y-2">
						{fetching ? (
							<div className="text-sm text-zinc-400">Loading collections...</div>
						) : manualCollections.length > 0 ? (
							manualCollections.map((collection) => (
								<button
									key={collection.id}
									type="button"
									className="flex w-full items-center justify-between rounded-md border border-zinc-800 px-3 py-3 text-left hover:bg-zinc-900"
									onClick={() => {
										void handleAdd(collection.id);
									}}
									disabled={adding || creating}
								>
									<span className="font-medium">{collection.name}</span>
									<PlusIcon className="size-4 text-zinc-500" />
								</button>
							))
						) : (
							<div className="text-sm text-zinc-400">No editable manual collections yet.</div>
						)}
					</div>
				</div>

				<form className="space-y-3" onSubmit={(event) => void handleCreate(event)}>
					<div className="text-xs font-medium uppercase tracking-wide text-zinc-400">Create New</div>
					<div className="flex items-center gap-2">
						<Input
							placeholder="Late Night Picks"
							value={newCollectionName}
							onChange={(event) => setNewCollectionName(event.target.value)}
						/>
						<Button type="submit" style={ButtonStyle.White} disabled={!newCollectionName.trim() || creating || adding}>
							<FolderPlusIcon className="size-5" />
						</Button>
					</div>
				</form>

				{error ? <div className="rounded bg-red-950/50 px-3 py-2 text-sm text-red-300">{error}</div> : null}
			</ModalBody>
			<ModalFooter>
				<Button style={ButtonStyle.Transparent} onClick={() => onOpenChange(false)}>
					Close
				</Button>
			</ModalFooter>
		</Modal>
	);
};
