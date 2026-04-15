import type { Cache, UpdatesConfig } from "@urql/exchange-graphcache";
import type {
  CreateLibraryMutation,
  CreatePrivateCollectionMutation,
  CreateUserInviteMutation,
  DeleteLibraryMutation,
  DeleteCollectionMutation,
  DeleteUserMutation,
  AddNodeToCollectionMutation,
  AddNodeToWatchlistMutation,
  RunImportWatchStatesMutation,
  RunImportWatchStatesMutationVariables,
  RemoveNodeFromWatchlistMutation,
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
  invalidateQueryField(cache, "home");
};

const updateCreateCollection = (
  _result: CreatePrivateCollectionMutation,
  _args: { name: string; resolverKind: string; visibility: string },
  cache: Cache,
) => {
  invalidateQueryField(cache, "collections");
  invalidateQueryField(cache, "home");
};

const updateDeleteCollection = (_result: DeleteCollectionMutation, _args: { collectionId: string }, cache: Cache) => {
  invalidateQueryField(cache, "collections");
  invalidateQueryField(cache, "collection");
  invalidateQueryField(cache, "home");
};

const updateAddNodeToCollection = (
  _result: AddNodeToCollectionMutation,
  _args: { collectionId: string; nodeId: string },
  cache: Cache,
) => {
  invalidateQueryField(cache, "collections");
  invalidateQueryField(cache, "collection");
  invalidateQueryField(cache, "home");
};

const updateWatchlist = (
  _result: AddNodeToWatchlistMutation | RemoveNodeFromWatchlistMutation,
  _args: { nodeId: string },
  cache: Cache,
) => {
  invalidateQueryField(cache, "home");
  invalidateQueryField(cache, "collection");
  invalidateQueryField(cache, "collections");
  invalidateQueryField(cache, "node");
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
    createCollection: updateCreateCollection,
    createUserInvite: updateCreateUserInvite,
    addNodeToCollection: updateAddNodeToCollection,
    addNodeToWatchlist: updateWatchlist,
    deleteCollection: updateDeleteCollection,
    deleteLibrary: updateDeleteLibrary,
    deleteUser: updateDeleteUser,
    importWatchStates: updateImportWatchStates,
    removeNodeFromWatchlist: updateWatchlist,
  },
} satisfies UpdatesConfig;
