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
    "\n\tsubscription ContentUpdates {\n\t\tcontentUpdates\n\t}\n": typeof types.ContentUpdatesDocument,
    "\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n": typeof types.GetFilesDocument,
    "\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n": typeof types.EpisodeCardFragmentDoc,
    "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tsignedUrl\n\t\tthumbhash\n\t}\n": typeof types.ImageAssetFragmentDoc,
    "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n": typeof types.RunImportWatchStatesDocument,
    "\n\tfragment NodeList on Node {\n\t\tid\n\t\t...NodePoster\n\t}\n": typeof types.NodeListFragmentDoc,
    "\n\tfragment NodePoster on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": typeof types.NodePosterFragmentDoc,
    "\n\tfragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n\t\tpositionMs\n\t\tendMs\n\t\tsheetIntervalMs\n\t\tsheetGapSize\n\t\tasset {\n\t\t\tid\n\t\t\tsignedUrl\n\t\t\twidth\n\t\t\theight\n\t\t}\n\t}\n": typeof types.PlayerTimelinePreviewSheetFragmentDoc,
    "\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t}\n\t\t\troot {\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\ttracks {\n\t\t\t\t\ttrackIndex\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tdisplayName\n\t\t\t\t\tlanguage\n\t\t\t\t\tdisposition\n\t\t\t\t\tisForced\n\t\t\t\t}\n\t\t\t\trecommendedTracks {\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tenabled\n\t\t\t\t}\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\t...PlayerTimelinePreviewSheet\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n": typeof types.ItemPlaybackDocument,
    "\n\tmutation UpdateWatchState($fileId: String!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n": typeof types.UpdateWatchStateDocument,
    "\n\tmutation SetPreferredAudio($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredAudio(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredAudioLanguage\n\t\t\tpreferredAudioDisposition\n\t\t}\n\t}\n": typeof types.SetPreferredAudioDocument,
    "\n\tmutation SetPreferredSubtitle($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredSubtitle(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredSubtitleLanguage\n\t\t\tpreferredSubtitleDisposition\n\t\t}\n\t}\n": typeof types.SetPreferredSubtitleDocument,
    "\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t}\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n": typeof types.SearchNodeResultFragmentDoc,
    "\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n": typeof types.SearchMediaDocument,
    "\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": typeof types.SeasonCardFragmentDoc,
    "\n\tfragment LibraryCard on Library {\n\t\tid\n\t\tname\n\t\tpath\n\t\tcreatedAt\n\t\tlastScannedAt\n\t}\n": typeof types.LibraryCardFragmentDoc,
    "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\t...LibraryCard\n\t\t}\n\t}\n": typeof types.GetLibrariesDocument,
    "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n": typeof types.CreateLibraryDocument,
    "\n\tmutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!) {\n\t\tupdateLibrary(libraryId: $libraryId, name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n": typeof types.UpdateLibraryDocument,
    "\n\tmutation DeleteLibrary($libraryId: String!) {\n\t\tdeleteLibrary(libraryId: $libraryId)\n\t}\n": typeof types.DeleteLibraryDocument,
    "\n\tquery UsersManagement {\n\t\tviewer {\n\t\t\tid\n\t\t}\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t\tusers {\n\t\t\tid\n\t\t\t...UserCard\n\t\t}\n\t}\n": typeof types.UsersManagementDocument,
    "\n\tmutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tcreateUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n": typeof types.CreateUserInviteDocument,
    "\n\tmutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tupdateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n": typeof types.UpdateUserDocument,
    "\n\tmutation ResetUserInvite($userId: String!) {\n\t\tresetUserInvite(userId: $userId) {\n\t\t\t...UserCard\n\t\t}\n\t}\n": typeof types.ResetUserInviteDocument,
    "\n\tmutation DeleteUser($userId: String!) {\n\t\tdeleteUser(userId: $userId)\n\t}\n": typeof types.DeleteUserDocument,
    "\n\tfragment UserCard on User {\n\t\tid\n\t\tusername\n\t\tinviteCode\n\t\tpermissions\n\t\tlibraries {\n\t\t\tid\n\t\t}\n\t\tcreatedAt\n\t\tlastSeenAt\n\t}\n": typeof types.UserCardFragmentDoc,
    "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n": typeof types.LibrariesDocument,
    "\n\tquery SidebarViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n": typeof types.SidebarViewerDocument,
    "\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n": typeof types.GetPathForNodeFragmentDoc,
    "\n\tquery GetAllNodes($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": typeof types.GetAllNodesDocument,
    "\n\tquery GetLibraryNodes($libraryId: String!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n": typeof types.GetLibraryNodesDocument,
    "\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\troot {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\tproperties {\n\t\t\t\t\tseasonNumber\n\t\t\t\t}\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t\tepisodeCount\n\t\t}\n\t}\n": typeof types.GetNodeByIdDocument,
    "\n\tquery GetEpisodes($filter: NodeFilter!, $first: Int) {\n\t\tnodeList(filter: $filter, first: $first) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n": typeof types.GetEpisodesDocument,
    "\n\tquery PlaygroundViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n": typeof types.PlaygroundViewerDocument,
    "\n\tquery SettingsViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n": typeof types.SettingsViewerDocument,
    "\n\tmutation Signup($username: String!, $password: String!, $inviteCode: String) {\n\t\tsignup(username: $username, password: $password, inviteCode: $inviteCode) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n": typeof types.SignupDocument,
};
const documents: Documents = {
    "\n\tquery GetActivities {\n\t\tactivities {\n\t\t\ttaskType\n\t\t\ttitle\n\t\t\tcurrent\n\t\t\ttotal\n\t\t\tprogressPercent\n\t\t}\n\t}\n": types.GetActivitiesDocument,
    "\n\tsubscription ContentUpdates {\n\t\tcontentUpdates\n\t}\n": types.ContentUpdatesDocument,
    "\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n": types.GetFilesDocument,
    "\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n": types.EpisodeCardFragmentDoc,
    "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tsignedUrl\n\t\tthumbhash\n\t}\n": types.ImageAssetFragmentDoc,
    "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n": types.RunImportWatchStatesDocument,
    "\n\tfragment NodeList on Node {\n\t\tid\n\t\t...NodePoster\n\t}\n": types.NodeListFragmentDoc,
    "\n\tfragment NodePoster on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": types.NodePosterFragmentDoc,
    "\n\tfragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n\t\tpositionMs\n\t\tendMs\n\t\tsheetIntervalMs\n\t\tsheetGapSize\n\t\tasset {\n\t\t\tid\n\t\t\tsignedUrl\n\t\t\twidth\n\t\t\theight\n\t\t}\n\t}\n": types.PlayerTimelinePreviewSheetFragmentDoc,
    "\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t}\n\t\t\troot {\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\ttracks {\n\t\t\t\t\ttrackIndex\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tdisplayName\n\t\t\t\t\tlanguage\n\t\t\t\t\tdisposition\n\t\t\t\t\tisForced\n\t\t\t\t}\n\t\t\t\trecommendedTracks {\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tenabled\n\t\t\t\t}\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\t...PlayerTimelinePreviewSheet\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n": types.ItemPlaybackDocument,
    "\n\tmutation UpdateWatchState($fileId: String!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n": types.UpdateWatchStateDocument,
    "\n\tmutation SetPreferredAudio($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredAudio(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredAudioLanguage\n\t\t\tpreferredAudioDisposition\n\t\t}\n\t}\n": types.SetPreferredAudioDocument,
    "\n\tmutation SetPreferredSubtitle($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredSubtitle(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredSubtitleLanguage\n\t\t\tpreferredSubtitleDisposition\n\t\t}\n\t}\n": types.SetPreferredSubtitleDocument,
    "\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t}\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n": types.SearchNodeResultFragmentDoc,
    "\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n": types.SearchMediaDocument,
    "\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n": types.SeasonCardFragmentDoc,
    "\n\tfragment LibraryCard on Library {\n\t\tid\n\t\tname\n\t\tpath\n\t\tcreatedAt\n\t\tlastScannedAt\n\t}\n": types.LibraryCardFragmentDoc,
    "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\t...LibraryCard\n\t\t}\n\t}\n": types.GetLibrariesDocument,
    "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n": types.CreateLibraryDocument,
    "\n\tmutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!) {\n\t\tupdateLibrary(libraryId: $libraryId, name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n": types.UpdateLibraryDocument,
    "\n\tmutation DeleteLibrary($libraryId: String!) {\n\t\tdeleteLibrary(libraryId: $libraryId)\n\t}\n": types.DeleteLibraryDocument,
    "\n\tquery UsersManagement {\n\t\tviewer {\n\t\t\tid\n\t\t}\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t\tusers {\n\t\t\tid\n\t\t\t...UserCard\n\t\t}\n\t}\n": types.UsersManagementDocument,
    "\n\tmutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tcreateUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n": types.CreateUserInviteDocument,
    "\n\tmutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tupdateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n": types.UpdateUserDocument,
    "\n\tmutation ResetUserInvite($userId: String!) {\n\t\tresetUserInvite(userId: $userId) {\n\t\t\t...UserCard\n\t\t}\n\t}\n": types.ResetUserInviteDocument,
    "\n\tmutation DeleteUser($userId: String!) {\n\t\tdeleteUser(userId: $userId)\n\t}\n": types.DeleteUserDocument,
    "\n\tfragment UserCard on User {\n\t\tid\n\t\tusername\n\t\tinviteCode\n\t\tpermissions\n\t\tlibraries {\n\t\t\tid\n\t\t}\n\t\tcreatedAt\n\t\tlastSeenAt\n\t}\n": types.UserCardFragmentDoc,
    "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n": types.LibrariesDocument,
    "\n\tquery SidebarViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n": types.SidebarViewerDocument,
    "\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n": types.GetPathForNodeFragmentDoc,
    "\n\tquery GetAllNodes($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n": types.GetAllNodesDocument,
    "\n\tquery GetLibraryNodes($libraryId: String!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n": types.GetLibraryNodesDocument,
    "\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\troot {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\tproperties {\n\t\t\t\t\tseasonNumber\n\t\t\t\t}\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t\tepisodeCount\n\t\t}\n\t}\n": types.GetNodeByIdDocument,
    "\n\tquery GetEpisodes($filter: NodeFilter!, $first: Int) {\n\t\tnodeList(filter: $filter, first: $first) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n": types.GetEpisodesDocument,
    "\n\tquery PlaygroundViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n": types.PlaygroundViewerDocument,
    "\n\tquery SettingsViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n": types.SettingsViewerDocument,
    "\n\tmutation Signup($username: String!, $password: String!, $inviteCode: String) {\n\t\tsignup(username: $username, password: $password, inviteCode: $inviteCode) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n": types.SignupDocument,
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
export function graphql(source: "\n\tsubscription ContentUpdates {\n\t\tcontentUpdates\n\t}\n"): (typeof documents)["\n\tsubscription ContentUpdates {\n\t\tcontentUpdates\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n"): (typeof documents)["\n\tquery GetFiles($path: String!) {\n\t\tlistFiles(path: $path)\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment EpisodeCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tdescription\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\twatchProgress {\n\t\t\tprogressPercent\n\t\t\tcompleted\n\t\t\tupdatedAt\n\t\t}\n\t\t...GetPathForNode\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tsignedUrl\n\t\tthumbhash\n\t}\n"): (typeof documents)["\n\tfragment ImageAsset on Asset {\n\t\tid\n\t\tsignedUrl\n\t\tthumbhash\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n\t\timportWatchStates(input: $input) {\n\t\t\tdryRun\n\t\t\ttotalRows\n\t\t\tmatchedRows\n\t\t\tunmatchedRows\n\t\t\tconflictRows\n\t\t\twillInsert\n\t\t\twillOverwrite\n\t\t\timported\n\t\t\tskipped\n\t\t\tconflicts {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\titemId\n\t\t\t\texistingProgressPercent\n\t\t\t\timportedProgressPercent\n\t\t\t\treason\n\t\t\t}\n\t\t\tunmatched {\n\t\t\t\trowIndex\n\t\t\t\tsourceItemId\n\t\t\t\ttitle\n\t\t\t\treason\n\t\t\t\tambiguous\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment NodeList on Node {\n\t\tid\n\t\t...NodePoster\n\t}\n"): (typeof documents)["\n\tfragment NodeList on Node {\n\t\tid\n\t\t...NodePoster\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment NodePoster on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment NodePoster on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tseasonCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n\t\tpositionMs\n\t\tendMs\n\t\tsheetIntervalMs\n\t\tsheetGapSize\n\t\tasset {\n\t\t\tid\n\t\t\tsignedUrl\n\t\t\twidth\n\t\t\theight\n\t\t}\n\t}\n"): (typeof documents)["\n\tfragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n\t\tpositionMs\n\t\tendMs\n\t\tsheetIntervalMs\n\t\tsheetGapSize\n\t\tasset {\n\t\t\tid\n\t\t\tsignedUrl\n\t\t\twidth\n\t\t\theight\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t}\n\t\t\troot {\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\ttracks {\n\t\t\t\t\ttrackIndex\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tdisplayName\n\t\t\t\t\tlanguage\n\t\t\t\t\tdisposition\n\t\t\t\t\tisForced\n\t\t\t\t}\n\t\t\t\trecommendedTracks {\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tenabled\n\t\t\t\t}\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\t...PlayerTimelinePreviewSheet\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery ItemPlayback($itemId: String!) {\n\t\tnode(nodeId: $itemId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tseasonNumber\n\t\t\t\tepisodeNumber\n\t\t\t\truntimeMinutes\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t}\n\t\t\troot {\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tfile {\n\t\t\t\tid\n\t\t\t\ttracks {\n\t\t\t\t\ttrackIndex\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tdisplayName\n\t\t\t\t\tlanguage\n\t\t\t\t\tdisposition\n\t\t\t\t\tisForced\n\t\t\t\t}\n\t\t\t\trecommendedTracks {\n\t\t\t\t\tmanifestIndex\n\t\t\t\t\ttrackType\n\t\t\t\t\tenabled\n\t\t\t\t}\n\t\t\t\tsegments {\n\t\t\t\t\tkind\n\t\t\t\t\tstartMs\n\t\t\t\t\tendMs\n\t\t\t\t}\n\t\t\t\ttimelinePreview {\n\t\t\t\t\t...PlayerTimelinePreviewSheet\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t\tdescription\n\t\t\t\t\tthumbnailImage {\n\t\t\t\t\t\t...ImageAsset\n\t\t\t\t\t}\n\t\t\t\t\tseasonNumber\n\t\t\t\t\tepisodeNumber\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation UpdateWatchState($fileId: String!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation UpdateWatchState($fileId: String!, $progressPercent: Float!) {\n\t\tupdateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {\n\t\t\tprogressPercent\n\t\t\tupdatedAt\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation SetPreferredAudio($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredAudio(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredAudioLanguage\n\t\t\tpreferredAudioDisposition\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation SetPreferredAudio($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredAudio(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredAudioLanguage\n\t\t\tpreferredAudioDisposition\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation SetPreferredSubtitle($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredSubtitle(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredSubtitleLanguage\n\t\t\tpreferredSubtitleDisposition\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation SetPreferredSubtitle($language: String, $disposition: TrackDispositionPreference) {\n\t\tsetPreferredSubtitle(language: $language, disposition: $disposition) {\n\t\t\tid\n\t\t\tpreferredSubtitleLanguage\n\t\t\tpreferredSubtitleDisposition\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t}\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment SearchNodeResult on Node {\n\t\tid\n\t\tkind\n\t\tlibraryId\n\t\troot {\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t}\n\t\t}\n\t\tseasonCount\n\t\tepisodeCount\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tdescription\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t\truntimeMinutes\n\t\t}\n\t\t...GetPathForNode\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery SearchMedia($query: String!, $limit: Int) {\n\t\tsearch(query: $query, limit: $limit) {\n\t\t\troots {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t\tepisodes {\n\t\t\t\t...SearchNodeResult\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"): (typeof documents)["\n\tfragment SeasonCard on Node {\n\t\tid\n\t\tproperties {\n\t\t\tdisplayName\n\t\t\tseasonNumber\n\t\t\tposterImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\tthumbnailImage {\n\t\t\t\t...ImageAsset\n\t\t\t}\n\t\t\treleasedAt\n\t\t\tendedAt\n\t\t}\n\t\tnextPlayable {\n\t\t\tid\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t}\n\t\tunplayedCount\n\t\tepisodeCount\n\t\t...GetPathForNode\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment LibraryCard on Library {\n\t\tid\n\t\tname\n\t\tpath\n\t\tcreatedAt\n\t\tlastScannedAt\n\t}\n"): (typeof documents)["\n\tfragment LibraryCard on Library {\n\t\tid\n\t\tname\n\t\tpath\n\t\tcreatedAt\n\t\tlastScannedAt\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\t...LibraryCard\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetLibraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\t...LibraryCard\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation CreateLibrary($name: String!, $path: String!) {\n\t\tcreateLibrary(name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!) {\n\t\tupdateLibrary(libraryId: $libraryId, name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!) {\n\t\tupdateLibrary(libraryId: $libraryId, name: $name, path: $path) {\n\t\t\t...LibraryCard\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation DeleteLibrary($libraryId: String!) {\n\t\tdeleteLibrary(libraryId: $libraryId)\n\t}\n"): (typeof documents)["\n\tmutation DeleteLibrary($libraryId: String!) {\n\t\tdeleteLibrary(libraryId: $libraryId)\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery UsersManagement {\n\t\tviewer {\n\t\t\tid\n\t\t}\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t\tusers {\n\t\t\tid\n\t\t\t...UserCard\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery UsersManagement {\n\t\tviewer {\n\t\t\tid\n\t\t}\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t\tusers {\n\t\t\tid\n\t\t\t...UserCard\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tcreateUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tcreateUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tupdateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n\t\tupdateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n\t\t\t...UserCard\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation ResetUserInvite($userId: String!) {\n\t\tresetUserInvite(userId: $userId) {\n\t\t\t...UserCard\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation ResetUserInvite($userId: String!) {\n\t\tresetUserInvite(userId: $userId) {\n\t\t\t...UserCard\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation DeleteUser($userId: String!) {\n\t\tdeleteUser(userId: $userId)\n\t}\n"): (typeof documents)["\n\tmutation DeleteUser($userId: String!) {\n\t\tdeleteUser(userId: $userId)\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment UserCard on User {\n\t\tid\n\t\tusername\n\t\tinviteCode\n\t\tpermissions\n\t\tlibraries {\n\t\t\tid\n\t\t}\n\t\tcreatedAt\n\t\tlastSeenAt\n\t}\n"): (typeof documents)["\n\tfragment UserCard on User {\n\t\tid\n\t\tusername\n\t\tinviteCode\n\t\tpermissions\n\t\tlibraries {\n\t\t\tid\n\t\t}\n\t\tcreatedAt\n\t\tlastSeenAt\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery Libraries {\n\t\tlibraries {\n\t\t\tid\n\t\t\tname\n\t\t\tcreatedAt\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery SidebarViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery SidebarViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n"): (typeof documents)["\n\tfragment GetPathForNode on Node {\n\t\tid\n\t\tlibraryId\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetAllNodes($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetAllNodes($filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetLibraryNodes($libraryId: String!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetLibraryNodes($libraryId: String!, $filter: NodeFilter!, $after: String) {\n\t\tnodeList(filter: $filter, first: 45, after: $after) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...NodeList\n\t\t\t\t}\n\t\t\t}\n\t\t\tpageInfo {\n\t\t\t\tendCursor\n\t\t\t\thasNextPage\n\t\t\t}\n\t\t}\n\t\tlibrary(libraryId: $libraryId) {\n\t\t\tid\n\t\t\tname\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\troot {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\tproperties {\n\t\t\t\t\tseasonNumber\n\t\t\t\t}\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t\tepisodeCount\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetNodeById($nodeId: String!) {\n\t\tnode(nodeId: $nodeId) {\n\t\t\tid\n\t\t\tlibraryId\n\t\t\tkind\n\t\t\tseasonNumber\n\t\t\tepisodeNumber\n\t\t\tparent {\n\t\t\t\tid\n\t\t\t\tlibraryId\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\troot {\n\t\t\t\tid\n\t\t\t\tproperties {\n\t\t\t\t\tdisplayName\n\t\t\t\t}\n\t\t\t}\n\t\t\tchildren {\n\t\t\t\tid\n\t\t\t\tkind\n\t\t\t\torder\n\t\t\t\tproperties {\n\t\t\t\t\tseasonNumber\n\t\t\t\t}\n\t\t\t\t...SeasonCard\n\t\t\t\t...EpisodeCard\n\t\t\t}\n\t\t\tproperties {\n\t\t\t\tdisplayName\n\t\t\t\tposterImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tbackgroundImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\tthumbnailImage {\n\t\t\t\t\t...ImageAsset\n\t\t\t\t}\n\t\t\t\treleasedAt\n\t\t\t\tendedAt\n\t\t\t\truntimeMinutes\n\t\t\t\tdescription\n\t\t\t}\n\t\t\twatchProgress {\n\t\t\t\tprogressPercent\n\t\t\t\tcompleted\n\t\t\t\tupdatedAt\n\t\t\t}\n\t\t\tnextPlayable {\n\t\t\t\tid\n\t\t\t\twatchProgress {\n\t\t\t\t\tprogressPercent\n\t\t\t\t\tcompleted\n\t\t\t\t\tupdatedAt\n\t\t\t\t}\n\t\t\t}\n\t\t\tpreviousPlayable {\n\t\t\t\tid\n\t\t\t}\n\t\t\tunplayedCount\n\t\t\tepisodeCount\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery GetEpisodes($filter: NodeFilter!, $first: Int) {\n\t\tnodeList(filter: $filter, first: $first) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery GetEpisodes($filter: NodeFilter!, $first: Int) {\n\t\tnodeList(filter: $filter, first: $first) {\n\t\t\tedges {\n\t\t\t\tnode {\n\t\t\t\t\tid\n\t\t\t\t\t...EpisodeCard\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery PlaygroundViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery PlaygroundViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tquery SettingsViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n"): (typeof documents)["\n\tquery SettingsViewer {\n\t\tviewer {\n\t\t\tid\n\t\t\tpermissions\n\t\t}\n\t}\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n\tmutation Signup($username: String!, $password: String!, $inviteCode: String) {\n\t\tsignup(username: $username, password: $password, inviteCode: $inviteCode) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n"): (typeof documents)["\n\tmutation Signup($username: String!, $password: String!, $inviteCode: String) {\n\t\tsignup(username: $username, password: $password, inviteCode: $inviteCode) {\n\t\t\tid\n\t\t\tusername\n\t\t}\n\t}\n"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;