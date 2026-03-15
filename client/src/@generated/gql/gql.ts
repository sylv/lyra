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
    "\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n": typeof types.GetFilesDocument,
    "\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n": typeof types.EpisodeCardFragmentDoc,
    "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tthumbhash\n\t}\n": typeof types.ImageAssetFragmentDoc,
    "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n": typeof types.RunImportWatchStatesDocument,
    "\n\tfragment MediaList on Node {\n\t\tid\n\t\t...MediaPoster\n\t}\n": typeof types.MediaListFragmentDoc,
    "\n\tfragment MediaPoster on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": typeof types.MediaPosterFragmentDoc,
    "\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n": typeof types.UpdateWatchStateDocument,
    "\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\troot {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n": typeof types.ItemPlaybackDocument,
    "\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tname\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n": typeof types.SearchNodeResultFragmentDoc,
    "\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n": typeof types.SearchMediaDocument,
    "\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": typeof types.SeasonCardFragmentDoc,
    "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": typeof types.GetLibrariesDocument,
    "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": typeof types.CreateLibraryDocument,
    "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n": typeof types.LibrariesDocument,
    "\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n": typeof types.GetPathForNodeFragmentDoc,
    "\n\tquery GetAllMedia($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": typeof types.GetAllMediaDocument,
    "\n\tquery GetLibraryMedia($libraryId: Int!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n": typeof types.GetLibraryMediaDocument,
    "\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t}\n\t}\n": typeof types.GetNodeByIdDocument,
    "\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n": typeof types.SignupDocument,
};
const documents: Documents = {
    "\n\tquery GetActivities {\n\t\tactivities {\n\t\t\ttaskType\n\t\t\ttitle\n\t\t\tcurrent\n\t\t\ttotal\n\t\t\tprogressPercent\n\t\t}\n\t}\n": types.GetActivitiesDocument,
    "\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n": types.GetFilesDocument,
    "\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n": types.EpisodeCardFragmentDoc,
    "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tthumbhash\n\t}\n": types.ImageAssetFragmentDoc,
    "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n": types.RunImportWatchStatesDocument,
    "\n\tfragment MediaList on Node {\n\t\tid\n\t\t...MediaPoster\n\t}\n": types.MediaListFragmentDoc,
    "\n\tfragment MediaPoster on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": types.MediaPosterFragmentDoc,
    "\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n": types.UpdateWatchStateDocument,
    "\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\troot {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n": types.ItemPlaybackDocument,
    "\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tname\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n": types.SearchNodeResultFragmentDoc,
    "\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n": types.SearchMediaDocument,
    "\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": types.SeasonCardFragmentDoc,
    "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": types.GetLibrariesDocument,
    "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\tid\n\t\t\tname\n\t\t\tpath\n\t\t}\n\t}\n": types.CreateLibraryDocument,
    "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n": types.LibrariesDocument,
    "\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n": types.GetPathForNodeFragmentDoc,
    "\n\tquery GetAllMedia($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": types.GetAllMediaDocument,
    "\n\tquery GetLibraryMedia($libraryId: Int!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n": types.GetLibraryMediaDocument,
    "\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t}\n\t}\n": types.GetNodeByIdDocument,
    "\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n": types.SignupDocument,
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
export function graphql(source: "\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n"): (typeof documents)["\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n"];
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
export function graphql(source: "\n\tfragment MediaList on Node {\n\t\tid\n\t\t...MediaPoster\n\t}\n"): (typeof documents)["\n\tfragment MediaList on Node {\n\t\tid\n\t\t...MediaPoster\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment MediaPoster on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment MediaPoster on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation UpdateWatchState($fileId: Int!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\troot {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tproperties {\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t}\n\t\t\troot {\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\tpositionMs\n\t\t\t\t\tendMs\n\t\t\t\t\tsheetIntervalMs\n\t\t\t\t\tsheetGapSize\n\t\t\t\t\tasset {\n\t\t\t\t\t\tid\n\t\t\t\t\t\twidth\n\t\t\t\t\t\theight\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tname\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tname\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tname\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tname\n\t\tproperties {\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"];
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
export function graphql(source: "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n"): (typeof documents)["\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetAllMedia($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetAllMedia($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetLibraryMedia($libraryId: Int!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetLibraryMedia($libraryId: Int!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...MediaList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tname\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tname\n\t\t\t\tlibraryId\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation Signup($username: String!, $password: String!) {\n\t\tsignup(username: $username, password: $password) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;