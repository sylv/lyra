import type { Cache, UpdatesConfig } from "@urql/exchange-graphcache";
import type {
	CreateLibraryMutation,
	CreateUserInviteMutation,
	DeleteLibraryMutation,
	DeleteUserMutation,
	RunImportWatchStatesMutation,
	RunImportWatchStatesMutationVariables,
} from "./@generated/gql/graphql";

const invalidateQueryField = (cache: Cache, fieldName: string) => {
	for (const field of cache.inspectFields("Query")) {
		if (field.fieldName === fieldName) {
			cache.invalidate("Query", field.fieldName, field.arguments ?? undefined);
		}
	}
};

const updateCreateUserInvite = (
	_result: CreateUserInviteMutation,
	_args: { username: string; permissions: number; libraryIds: string[] },
	cache: Cache,
) => {
	invalidateQueryField(cache, "users");
};

const updateDeleteUser = (_result: DeleteUserMutation, _args: { userId?: string | null }, cache: Cache) => {
	invalidateQueryField(cache, "users");
};

const updateCreateLibrary = (_result: CreateLibraryMutation, _args: { name: string; path: string }, cache: Cache) => {
	invalidateQueryField(cache, "libraries");
};

const updateDeleteLibrary = (_result: DeleteLibraryMutation, _args: { libraryId?: string | null }, cache: Cache) => {
	invalidateQueryField(cache, "libraries");
};

const updateImportWatchStates = (
	_result: RunImportWatchStatesMutation,
	args: RunImportWatchStatesMutationVariables,
	cache: Cache,
) => {
	if (args.input.dryRun) {
		return;
	}

	// Watch-state imports can affect any cached node detail or paginated node list.
	invalidateQueryField(cache, "node");
	invalidateQueryField(cache, "nodeList");
};

export const cacheUpdates = {
	Mutation: {
		createLibrary: updateCreateLibrary,
		createUserInvite: updateCreateUserInvite,
		deleteLibrary: updateDeleteLibrary,
		deleteUser: updateDeleteUser,
		importWatchStates: updateImportWatchStates,
	},
} satisfies UpdatesConfig;
