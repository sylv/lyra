import { graphql } from "../../@generated/gql";

export const LibrariesQuery = graphql(`
	query GetLibraries {
		libraries {
			id
			...LibraryCard
		}
	}
`);

export const CreateLibraryMutation = graphql(`
	mutation CreateLibrary($name: String!, $path: String!) {
		createLibrary(name: $name, path: $path) {
			...LibraryCard
		}
	}
`);

export const UpdateLibraryMutation = graphql(`
	mutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!) {
		updateLibrary(libraryId: $libraryId, name: $name, path: $path) {
			...LibraryCard
		}
	}
`);

export const DeleteLibraryMutation = graphql(`
	mutation DeleteLibrary($libraryId: String!) {
		deleteLibrary(libraryId: $libraryId)
	}
`);
