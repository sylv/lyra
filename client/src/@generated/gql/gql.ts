/* eslint-disable */
import * as types from './graphql';
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
type Documents = {
    "\n\tquery GetActivities {\n\t\tactivities {\n\t\t\ttaskType\n\t\t\ttitle\n\t\t\tcurrent\n\t\t\ttotal\n\t\t\tprogressPercent\n\t\t}\n\t}\n": typeof types.GetActivitiesDocument,
    "\n    query GetFiles($path: String!) {\n        listFiles(path: $path)\n    }\n": typeof types.GetFilesDocument,
    "\n\tfragment EpisodeCard on ItemNode {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForItem\n\t}\n": typeof types.EpisodeCardFragmentDoc,
    "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tthumbhash\n\t}\n": typeof types.ImageAssetFragmentDoc,
    "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n": typeof types.RunImportWatchStatesDocument,
    "\n\tfragment MediaList on RootNode {\n\t\tid\n\t\t...MediaPoster\n\t}\n": typeof types.MediaListFragmentDoc,
    "\n\tfragment MediaPoster on RootNode {\n\t\tid\n\t\tname\n\t\tkind\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForRoot\n\t}\n": typeof types.MediaPosterFragmentDoc,
    "\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n": typeof types.UpdateWatchStateDocument,
    "\n\tquery ItemPlayback($itemId: String!) {\n\t\titem(itemId: $itemId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\trootId\n\t\t\tseasonId\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\tparent {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousItem {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n": typeof types.ItemPlaybackDocument,
    "\n\tfragment SeasonCard on SeasonNode {\n\t\tid\n\t\tname\n\t\tseasonNumber\n\t\torder\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tepisodeCount\n\t}\n": typeof types.SeasonCardFragmentDoc,
    "\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n": typeof types.SignupDocument,
    "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": typeof types.GetLibrariesDocument,
    "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": typeof types.CreateLibraryDocument,
    "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\t\n": typeof types.LibrariesDocument,
    "\n\tfragment GetPathForRoot on RootNode {\n\t\tid\n\t\tlibraryId\n\t}\n": typeof types.GetPathForRootFragmentDoc,
    "\n\tfragment GetPathForItem on ItemNode {\n\t\tkind\n\t\trootId\n\t\tseasonId\n\t\tparent {\n\t\t\tlibraryId\n\t\t}\n\t}\n": typeof types.GetPathForItemFragmentDoc,
    "\n\tquery GetAllMedia($filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": typeof types.GetAllMediaDocument,
    "\n\tquery GetLibraryMedia($libraryId: Int!, $filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n": typeof types.GetLibraryMediaDocument,
    "\n\tquery GetRootById($rootId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\tlibraryId\n\t\t\tseasons {\n\t\t\t\tid\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n": typeof types.GetRootByIdDocument,
    "\n\tquery GetRootAndSeason($rootId: String!, $seasonId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t\tseason(seasonId: $seasonId) {\n\t\t\tid\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n": typeof types.GetRootAndSeasonDocument,
    "\n\tquery GetSeasonEpisodes($filter: ItemNodeFilter!, $after: String) {\n\t\titemList(filter: $filter, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": typeof types.GetSeasonEpisodesDocument,
};
const documents: Documents = {
    "\n\tquery GetActivities {\n\t\tactivities {\n\t\t\ttaskType\n\t\t\ttitle\n\t\t\tcurrent\n\t\t\ttotal\n\t\t\tprogressPercent\n\t\t}\n\t}\n": types.GetActivitiesDocument,
    "\n    query GetFiles($path: String!) {\n        listFiles(path: $path)\n    }\n": types.GetFilesDocument,
    "\n\tfragment EpisodeCard on ItemNode {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForItem\n\t}\n": types.EpisodeCardFragmentDoc,
    "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tthumbhash\n\t}\n": types.ImageAssetFragmentDoc,
    "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n": types.RunImportWatchStatesDocument,
    "\n\tfragment MediaList on RootNode {\n\t\tid\n\t\t...MediaPoster\n\t}\n": types.MediaListFragmentDoc,
    "\n\tfragment MediaPoster on RootNode {\n\t\tid\n\t\tname\n\t\tkind\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForRoot\n\t}\n": types.MediaPosterFragmentDoc,
    "\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n": types.UpdateWatchStateDocument,
    "\n\tquery ItemPlayback($itemId: String!) {\n\t\titem(itemId: $itemId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\trootId\n\t\t\tseasonId\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\tparent {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousItem {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n": types.ItemPlaybackDocument,
    "\n\tfragment SeasonCard on SeasonNode {\n\t\tid\n\t\tname\n\t\tseasonNumber\n\t\torder\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tepisodeCount\n\t}\n": types.SeasonCardFragmentDoc,
    "\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n": types.SignupDocument,
    "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": types.GetLibrariesDocument,
    "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": types.CreateLibraryDocument,
    "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\t\n": types.LibrariesDocument,
    "\n\tfragment GetPathForRoot on RootNode {\n\t\tid\n\t\tlibraryId\n\t}\n": types.GetPathForRootFragmentDoc,
    "\n\tfragment GetPathForItem on ItemNode {\n\t\tkind\n\t\trootId\n\t\tseasonId\n\t\tparent {\n\t\t\tlibraryId\n\t\t}\n\t}\n": types.GetPathForItemFragmentDoc,
    "\n\tquery GetAllMedia($filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": types.GetAllMediaDocument,
    "\n\tquery GetLibraryMedia($libraryId: Int!, $filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n": types.GetLibraryMediaDocument,
    "\n\tquery GetRootById($rootId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\tlibraryId\n\t\t\tseasons {\n\t\t\t\tid\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n": types.GetRootByIdDocument,
    "\n\tquery GetRootAndSeason($rootId: String!, $seasonId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t\tseason(seasonId: $seasonId) {\n\t\t\tid\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n": types.GetRootAndSeasonDocument,
    "\n\tquery GetSeasonEpisodes($filter: ItemNodeFilter!, $after: String) {\n\t\titemList(filter: $filter, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": types.GetSeasonEpisodesDocument,
};

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = graphql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function graphql(source: string): unknown;

/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetActivities {\n\t\tactivities {\n\t\t\ttaskType\n\t\t\ttitle\n\t\t\tcurrent\n\t\t\ttotal\n\t\t\tprogressPercent\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetActivities {\n\t\tactivities {\n\t\t\ttaskType\n\t\t\ttitle\n\t\t\tcurrent\n\t\t\ttotal\n\t\t\tprogressPercent\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n    query GetFiles($path: String!) {\n        listFiles(path: $path)\n    }\n"): (typeof documents)["\n    query GetFiles($path: String!) {\n        listFiles(path: $path)\n    }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment EpisodeCard on ItemNode {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForItem\n\t}\n"): (typeof documents)["\n\tfragment EpisodeCard on ItemNode {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForItem\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tthumbhash\n\t}\n"): (typeof documents)["\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tthumbhash\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment MediaList on RootNode {\n\t\tid\n\t\t...MediaPoster\n\t}\n"): (typeof documents)["\n\tfragment MediaList on RootNode {\n\t\tid\n\t\t...MediaPoster\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment MediaPoster on RootNode {\n\t\tid\n\t\tname\n\t\tkind\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForRoot\n\t}\n"): (typeof documents)["\n\tfragment MediaPoster on RootNode {\n\t\tid\n\t\tname\n\t\tkind\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForRoot\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery ItemPlayback($itemId: String!) {\n\t\titem(itemId: $itemId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\trootId\n\t\t\tseasonId\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\tparent {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousItem {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery ItemPlayback($itemId: String!) {\n\t\titem(itemId: $itemId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\trootId\n\t\t\tseasonId\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\tparent {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousItem {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment SeasonCard on SeasonNode {\n\t\tid\n\t\tname\n\t\tseasonNumber\n\t\torder\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tepisodeCount\n\t}\n"): (typeof documents)["\n\tfragment SeasonCard on SeasonNode {\n\t\tid\n\t\tname\n\t\tseasonNumber\n\t\torder\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextItem {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedItems\n\t\tepisodeCount\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\t\n"): (typeof documents)["\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\t\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment GetPathForRoot on RootNode {\n\t\tid\n\t\tlibraryId\n\t}\n"): (typeof documents)["\n\tfragment GetPathForRoot on RootNode {\n\t\tid\n\t\tlibraryId\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment GetPathForItem on ItemNode {\n\t\tkind\n\t\trootId\n\t\tseasonId\n\t\tparent {\n\t\t\tlibraryId\n\t\t}\n\t}\n"): (typeof documents)["\n\tfragment GetPathForItem on ItemNode {\n\t\tkind\n\t\trootId\n\t\tseasonId\n\t\tparent {\n\t\t\tlibraryId\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetAllMedia($filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetAllMedia($filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetLibraryMedia($libraryId: Int!, $filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetLibraryMedia($libraryId: Int!, $filter: RootNodeFilter!, $after: String) {\n\t\trootList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetRootById($rootId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\tlibraryId\n\t\t\tseasons {\n\t\t\t\tid\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetRootById($rootId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tkind\n\t\t\tname\n\t\t\tlibraryId\n\t\t\tseasons {\n\t\t\t\tid\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetRootAndSeason($rootId: String!, $seasonId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t\tseason(seasonId: $seasonId) {\n\t\t\tid\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetRootAndSeason($rootId: String!, $seasonId: String!) {\n\t\troot(rootId: $rootId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t\tseason(seasonId: $seasonId) {\n\t\t\tid\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextItem {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tunplayedItems\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetSeasonEpisodes($filter: ItemNodeFilter!, $after: String) {\n\t\titemList(filter: $filter, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetSeasonEpisodes($filter: ItemNodeFilter!, $after: String) {\n\t\titemList(filter: $filter, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;