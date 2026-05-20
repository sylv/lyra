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
    "\n  query GetActivities {\n    activities {\n      taskType\n      title\n      current\n      total\n      progressPercent\n    }\n  }\n": typeof types.GetActivitiesDocument,
    "\n  query EditableCollections {\n    collections {\n      id\n      name\n      canEdit\n      resolverKind\n    }\n  }\n": typeof types.EditableCollectionsDocument,
    "\n  mutation CreatePrivateCollection(\n    $name: String!\n    $resolverKind: CollectionResolverKind!\n    $visibility: CollectionVisibility!\n  ) {\n    createCollection(name: $name, resolverKind: $resolverKind, visibility: $visibility) {\n      id\n      name\n    }\n  }\n": typeof types.CreatePrivateCollectionDocument,
    "\n  mutation AddNodeToCollection($collectionId: String!, $nodeId: String!) {\n    addNodeToCollection(collectionId: $collectionId, nodeId: $nodeId) {\n      id\n      name\n    }\n  }\n": typeof types.AddNodeToCollectionDocument,
    "\n  fragment CollectionShelf on Collection {\n    id\n    name\n    itemCount\n    nodeList(first: 12) {\n      nodes {\n        id\n        ...NodePoster\n      }\n      pageInfo {\n        hasNextPage\n      }\n    }\n  }\n": typeof types.CollectionShelfFragmentDoc,
    "\n  subscription ContentUpdates {\n    contentUpdates\n  }\n": typeof types.ContentUpdatesDocument,
    "\n  query GetFiles($path: String!) {\n    listFiles(path: $path)\n  }\n": typeof types.GetFilesDocument,
    "\n  fragment ImageAsset on Asset {\n    id\n    signedUrl\n    thumbhash\n  }\n": typeof types.ImageAssetFragmentDoc,
    "\n  mutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n    importWatchStates(input: $input) {\n      dryRun\n      totalRows\n      matchedRows\n      unmatchedRows\n      conflictRows\n      willInsert\n      willOverwrite\n      imported\n      skipped\n      conflicts {\n        rowIndex\n        sourceItemId\n        title\n        itemId\n        existingProgressPercent\n        importedProgressPercent\n        reason\n      }\n      unmatched {\n        rowIndex\n        sourceItemId\n        title\n        reason\n        ambiguous\n      }\n    }\n  }\n": typeof types.RunImportWatchStatesDocument,
    "\n  fragment EpisodeCard on Node {\n    id\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n      firstAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    watchProgressHint\n    ...GetPathForNode\n  }\n": typeof types.EpisodeCardFragmentDoc,
    "\n  query NodePage($after: String, $first: Int!, $filter: NodeFilter!) {\n    nodeList(after: $after, first: $first, filter: $filter) {\n      edges {\n        node {\n          id\n          ...NodePoster\n          ...EpisodeCard\n        }\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n": typeof types.NodePageDocument,
    "\n  fragment NodePoster on Node {\n    id\n    kind\n    libraryId\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      displayDetail\n      posterImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    seasonCount\n    episodeCount\n    seasonNumber\n    episodeNumber\n    ...GetPathForNode\n  }\n": typeof types.NodePosterFragmentDoc,
    "\n  query Player($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      ...GetPathForNode\n      properties {\n        displayName\n        seasonNumber\n        episodeNumber\n      }\n      root {\n        properties {\n          displayName\n        }\n      }\n      defaultFile {\n        id\n        height\n        width\n        resumeHint {\n          startMs\n          updatedAt\n        }\n        probe {\n          durationSeconds\n          width\n          height\n        }\n        playback {\n          hlsUrlTemplate\n          video {\n            ...PlayerVideoTrack\n          }\n          audio {\n            ...PlayerAudioTrack\n          }\n          subtitles {\n            ...PlayerSubtitleTrack\n          }\n        }\n        timelinePreview {\n          ...PlayerTimelinePreviewSheet\n        }\n        segments {\n          kind\n          startMs\n          endMs\n        }\n      }\n      previousPlayable {\n        id\n        ...PlayerItemCard\n      }\n      nextPlayable {\n        id\n        ...PlayerItemCard\n      }\n    }\n  }\n": typeof types.PlayerDocument,
    "\n  fragment PlayerAudioTrack on PlaybackAudioTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    languageBcp47\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n": typeof types.PlayerAudioTrackFragmentDoc,
    "\n  fragment PlayerVideoTrack on PlaybackVideoTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n": typeof types.PlayerVideoTrackFragmentDoc,
    "\n  fragment PlayerSubtitleTrack on PlaybackSubtitleTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    kind\n    languageBcp47\n    renditions {\n      variantId\n      codec\n      displayInfo\n      signedUrl\n    }\n  }\n": typeof types.PlayerSubtitleTrackFragmentDoc,
    "\n  fragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n    positionMs\n    endMs\n    sheetIntervalMs\n    sheetGapSize\n    asset {\n      id\n      signedUrl\n      width\n      height\n    }\n  }\n": typeof types.PlayerTimelinePreviewSheetFragmentDoc,
    "\n  fragment PlayerItemCard on Node {\n    id\n    ...GetPathForNode\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n    }\n  }\n": typeof types.PlayerItemCardFragmentDoc,
    "\n  fragment SearchNodeResult on Node {\n    id\n    kind\n    libraryId\n    root {\n      properties {\n        displayName\n      }\n    }\n    seasonCount\n    episodeCount\n    properties {\n      displayName\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      description\n      seasonNumber\n      episodeNumber\n      firstAired\n      lastAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    ...GetPathForNode\n  }\n": typeof types.SearchNodeResultFragmentDoc,
    "\n  query SearchMedia($query: String!, $limit: Int!, $kinds: [NodeKind!]!) {\n    nodeList(first: $limit, filter: { searchTerm: $query, kinds: $kinds }) {\n      nodes {\n        ...SearchNodeResult\n      }\n    }\n  }\n": typeof types.SearchMediaDocument,
    "\n  fragment SeasonCard on Node {\n    id\n    unavailableAt\n    properties {\n      displayName\n      seasonNumber\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    episodeCount\n    ...GetPathForNode\n  }\n": typeof types.SeasonCardFragmentDoc,
    "\n  fragment LibraryCard on Library {\n    id\n    name\n    path\n    pinned\n    createdAt\n    lastScannedAt\n  }\n": typeof types.LibraryCardFragmentDoc,
    "\n  query GetLibraries {\n    libraries {\n      id\n      ...LibraryCard\n    }\n  }\n": typeof types.GetLibrariesDocument,
    "\n  mutation CreateLibrary($name: String!, $path: String!, $pinned: Boolean!) {\n    createLibrary(name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n": typeof types.CreateLibraryDocument,
    "\n  mutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!, $pinned: Boolean!) {\n    updateLibrary(libraryId: $libraryId, name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n": typeof types.UpdateLibraryDocument,
    "\n  mutation DeleteLibrary($libraryId: String!) {\n    deleteLibrary(libraryId: $libraryId)\n  }\n": typeof types.DeleteLibraryDocument,
    "\n  query UsersManagement {\n    viewer {\n      id\n    }\n    libraries {\n      id\n      name\n      createdAt\n    }\n    users {\n      id\n      ...UserCard\n    }\n  }\n": typeof types.UsersManagementDocument,
    "\n  mutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    createUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n": typeof types.CreateUserInviteDocument,
    "\n  mutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    updateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n": typeof types.UpdateUserDocument,
    "\n  mutation ResetUserInvite($userId: String!) {\n    resetUserInvite(userId: $userId) {\n      ...UserCard\n    }\n  }\n": typeof types.ResetUserInviteDocument,
    "\n  mutation DeleteUser($userId: String!) {\n    deleteUser(userId: $userId)\n  }\n": typeof types.DeleteUserDocument,
    "\n  fragment UserCard on User {\n    id\n    username\n    inviteCode\n    permissions\n    libraries {\n      id\n    }\n    createdAt\n    lastSeenAt\n  }\n": typeof types.UserCardFragmentDoc,
    "\n  query SidebarNavigation {\n    libraries {\n      id\n      name\n      createdAt\n      pinned\n    }\n    collections(pinned: true) {\n      id\n      name\n      createdById\n    }\n  }\n": typeof types.SidebarNavigationDocument,
    "\n  query SidebarViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n": typeof types.SidebarViewerDocument,
    "\n  mutation AddNodeToWatchlist($nodeId: String!) {\n    addNodeToWatchlist(nodeId: $nodeId)\n  }\n": typeof types.AddNodeToWatchlistDocument,
    "\n  mutation RemoveNodeFromWatchlist($nodeId: String!) {\n    removeNodeFromWatchlist(nodeId: $nodeId)\n  }\n": typeof types.RemoveNodeFromWatchlistDocument,
    "\n  fragment GetPathForNode on Node {\n    id\n    libraryId\n  }\n": typeof types.GetPathForNodeFragmentDoc,
    "\n  query CollectionPage($collectionId: String!, $after: String, $first: Int!) {\n    collection(collectionId: $collectionId) {\n      id\n      name\n      description\n      itemCount\n      canDelete\n      nodeList(after: $after, first: $first) {\n        nodes {\n          id\n          ...NodePoster\n        }\n        pageInfo {\n          endCursor\n          hasNextPage\n        }\n      }\n    }\n  }\n": typeof types.CollectionPageDocument,
    "\n  mutation DeleteCollection($collectionId: String!) {\n    deleteCollection(collectionId: $collectionId)\n  }\n": typeof types.DeleteCollectionDocument,
    "\n  query CollectionsIndex {\n    collections {\n      id\n      name\n      description\n      itemCount\n      visibility\n      createdBy {\n        username\n      }\n    }\n  }\n": typeof types.CollectionsIndexDocument,
    "\n  query HomeCollections {\n    home {\n      sections {\n        id\n        ...CollectionShelf\n      }\n    }\n  }\n": typeof types.HomeCollectionsDocument,
    "\n  query GetNodeById($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      kind\n      inWatchlist\n      unavailableAt\n      unplayedCount\n      seasonCount\n      seasonNumber\n      episodeNumber\n      ...GetPathForNode\n      parent {\n        id\n        ...GetPathForNode\n      }\n      root {\n        id\n        ...GetPathForNode\n        properties {\n          displayName\n        }\n      }\n      properties {\n        displayName\n        tagline\n        posterImage {\n          ...ImageAsset\n        }\n        thumbnailImage {\n          ...ImageAsset\n        }\n        logoImage {\n          id\n          signedUrl\n          aspectRatio\n        }\n        backdropImage {\n          ...ImageAsset\n          signedUrl\n          aspectRatio\n        }\n        description\n        contentRating {\n          rating\n        }\n        genres {\n          name\n        }\n        cast {\n          characterName\n          department\n          person {\n            id\n            name\n            profileImage {\n              ...ImageAsset\n            }\n          }\n        }\n      }\n      watchProgressHint\n      currentPlayable {\n        id\n        seasonNumber\n        episodeNumber\n        watchProgressHint\n      }\n      defaultFile {\n        id\n        probe {\n          runtimeMinutes\n          width\n          height\n          videoCodec\n          videoBitrate\n        }\n      }\n      recommendedNodes {\n        id\n        ...NodePoster\n      }\n    }\n  }\n": typeof types.GetNodeByIdDocument,
    "\n  query PlaygroundViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n": typeof types.PlaygroundViewerDocument,
    "\n  query SettingsViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n": typeof types.SettingsViewerDocument,
    "\n  mutation Signup($username: String!, $password: String!, $inviteCode: String) {\n    signup(username: $username, password: $password, inviteCode: $inviteCode) {\n      id\n      username\n    }\n  }\n": typeof types.SignupDocument,
};
const documents: Documents = {
    "\n  query GetActivities {\n    activities {\n      taskType\n      title\n      current\n      total\n      progressPercent\n    }\n  }\n": types.GetActivitiesDocument,
    "\n  query EditableCollections {\n    collections {\n      id\n      name\n      canEdit\n      resolverKind\n    }\n  }\n": types.EditableCollectionsDocument,
    "\n  mutation CreatePrivateCollection(\n    $name: String!\n    $resolverKind: CollectionResolverKind!\n    $visibility: CollectionVisibility!\n  ) {\n    createCollection(name: $name, resolverKind: $resolverKind, visibility: $visibility) {\n      id\n      name\n    }\n  }\n": types.CreatePrivateCollectionDocument,
    "\n  mutation AddNodeToCollection($collectionId: String!, $nodeId: String!) {\n    addNodeToCollection(collectionId: $collectionId, nodeId: $nodeId) {\n      id\n      name\n    }\n  }\n": types.AddNodeToCollectionDocument,
    "\n  fragment CollectionShelf on Collection {\n    id\n    name\n    itemCount\n    nodeList(first: 12) {\n      nodes {\n        id\n        ...NodePoster\n      }\n      pageInfo {\n        hasNextPage\n      }\n    }\n  }\n": types.CollectionShelfFragmentDoc,
    "\n  subscription ContentUpdates {\n    contentUpdates\n  }\n": types.ContentUpdatesDocument,
    "\n  query GetFiles($path: String!) {\n    listFiles(path: $path)\n  }\n": types.GetFilesDocument,
    "\n  fragment ImageAsset on Asset {\n    id\n    signedUrl\n    thumbhash\n  }\n": types.ImageAssetFragmentDoc,
    "\n  mutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n    importWatchStates(input: $input) {\n      dryRun\n      totalRows\n      matchedRows\n      unmatchedRows\n      conflictRows\n      willInsert\n      willOverwrite\n      imported\n      skipped\n      conflicts {\n        rowIndex\n        sourceItemId\n        title\n        itemId\n        existingProgressPercent\n        importedProgressPercent\n        reason\n      }\n      unmatched {\n        rowIndex\n        sourceItemId\n        title\n        reason\n        ambiguous\n      }\n    }\n  }\n": types.RunImportWatchStatesDocument,
    "\n  fragment EpisodeCard on Node {\n    id\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n      firstAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    watchProgressHint\n    ...GetPathForNode\n  }\n": types.EpisodeCardFragmentDoc,
    "\n  query NodePage($after: String, $first: Int!, $filter: NodeFilter!) {\n    nodeList(after: $after, first: $first, filter: $filter) {\n      edges {\n        node {\n          id\n          ...NodePoster\n          ...EpisodeCard\n        }\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n": types.NodePageDocument,
    "\n  fragment NodePoster on Node {\n    id\n    kind\n    libraryId\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      displayDetail\n      posterImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    seasonCount\n    episodeCount\n    seasonNumber\n    episodeNumber\n    ...GetPathForNode\n  }\n": types.NodePosterFragmentDoc,
    "\n  query Player($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      ...GetPathForNode\n      properties {\n        displayName\n        seasonNumber\n        episodeNumber\n      }\n      root {\n        properties {\n          displayName\n        }\n      }\n      defaultFile {\n        id\n        height\n        width\n        resumeHint {\n          startMs\n          updatedAt\n        }\n        probe {\n          durationSeconds\n          width\n          height\n        }\n        playback {\n          hlsUrlTemplate\n          video {\n            ...PlayerVideoTrack\n          }\n          audio {\n            ...PlayerAudioTrack\n          }\n          subtitles {\n            ...PlayerSubtitleTrack\n          }\n        }\n        timelinePreview {\n          ...PlayerTimelinePreviewSheet\n        }\n        segments {\n          kind\n          startMs\n          endMs\n        }\n      }\n      previousPlayable {\n        id\n        ...PlayerItemCard\n      }\n      nextPlayable {\n        id\n        ...PlayerItemCard\n      }\n    }\n  }\n": types.PlayerDocument,
    "\n  fragment PlayerAudioTrack on PlaybackAudioTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    languageBcp47\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n": types.PlayerAudioTrackFragmentDoc,
    "\n  fragment PlayerVideoTrack on PlaybackVideoTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n": types.PlayerVideoTrackFragmentDoc,
    "\n  fragment PlayerSubtitleTrack on PlaybackSubtitleTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    kind\n    languageBcp47\n    renditions {\n      variantId\n      codec\n      displayInfo\n      signedUrl\n    }\n  }\n": types.PlayerSubtitleTrackFragmentDoc,
    "\n  fragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n    positionMs\n    endMs\n    sheetIntervalMs\n    sheetGapSize\n    asset {\n      id\n      signedUrl\n      width\n      height\n    }\n  }\n": types.PlayerTimelinePreviewSheetFragmentDoc,
    "\n  fragment PlayerItemCard on Node {\n    id\n    ...GetPathForNode\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n    }\n  }\n": types.PlayerItemCardFragmentDoc,
    "\n  fragment SearchNodeResult on Node {\n    id\n    kind\n    libraryId\n    root {\n      properties {\n        displayName\n      }\n    }\n    seasonCount\n    episodeCount\n    properties {\n      displayName\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      description\n      seasonNumber\n      episodeNumber\n      firstAired\n      lastAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    ...GetPathForNode\n  }\n": types.SearchNodeResultFragmentDoc,
    "\n  query SearchMedia($query: String!, $limit: Int!, $kinds: [NodeKind!]!) {\n    nodeList(first: $limit, filter: { searchTerm: $query, kinds: $kinds }) {\n      nodes {\n        ...SearchNodeResult\n      }\n    }\n  }\n": types.SearchMediaDocument,
    "\n  fragment SeasonCard on Node {\n    id\n    unavailableAt\n    properties {\n      displayName\n      seasonNumber\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    episodeCount\n    ...GetPathForNode\n  }\n": types.SeasonCardFragmentDoc,
    "\n  fragment LibraryCard on Library {\n    id\n    name\n    path\n    pinned\n    createdAt\n    lastScannedAt\n  }\n": types.LibraryCardFragmentDoc,
    "\n  query GetLibraries {\n    libraries {\n      id\n      ...LibraryCard\n    }\n  }\n": types.GetLibrariesDocument,
    "\n  mutation CreateLibrary($name: String!, $path: String!, $pinned: Boolean!) {\n    createLibrary(name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n": types.CreateLibraryDocument,
    "\n  mutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!, $pinned: Boolean!) {\n    updateLibrary(libraryId: $libraryId, name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n": types.UpdateLibraryDocument,
    "\n  mutation DeleteLibrary($libraryId: String!) {\n    deleteLibrary(libraryId: $libraryId)\n  }\n": types.DeleteLibraryDocument,
    "\n  query UsersManagement {\n    viewer {\n      id\n    }\n    libraries {\n      id\n      name\n      createdAt\n    }\n    users {\n      id\n      ...UserCard\n    }\n  }\n": types.UsersManagementDocument,
    "\n  mutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    createUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n": types.CreateUserInviteDocument,
    "\n  mutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    updateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n": types.UpdateUserDocument,
    "\n  mutation ResetUserInvite($userId: String!) {\n    resetUserInvite(userId: $userId) {\n      ...UserCard\n    }\n  }\n": types.ResetUserInviteDocument,
    "\n  mutation DeleteUser($userId: String!) {\n    deleteUser(userId: $userId)\n  }\n": types.DeleteUserDocument,
    "\n  fragment UserCard on User {\n    id\n    username\n    inviteCode\n    permissions\n    libraries {\n      id\n    }\n    createdAt\n    lastSeenAt\n  }\n": types.UserCardFragmentDoc,
    "\n  query SidebarNavigation {\n    libraries {\n      id\n      name\n      createdAt\n      pinned\n    }\n    collections(pinned: true) {\n      id\n      name\n      createdById\n    }\n  }\n": types.SidebarNavigationDocument,
    "\n  query SidebarViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n": types.SidebarViewerDocument,
    "\n  mutation AddNodeToWatchlist($nodeId: String!) {\n    addNodeToWatchlist(nodeId: $nodeId)\n  }\n": types.AddNodeToWatchlistDocument,
    "\n  mutation RemoveNodeFromWatchlist($nodeId: String!) {\n    removeNodeFromWatchlist(nodeId: $nodeId)\n  }\n": types.RemoveNodeFromWatchlistDocument,
    "\n  fragment GetPathForNode on Node {\n    id\n    libraryId\n  }\n": types.GetPathForNodeFragmentDoc,
    "\n  query CollectionPage($collectionId: String!, $after: String, $first: Int!) {\n    collection(collectionId: $collectionId) {\n      id\n      name\n      description\n      itemCount\n      canDelete\n      nodeList(after: $after, first: $first) {\n        nodes {\n          id\n          ...NodePoster\n        }\n        pageInfo {\n          endCursor\n          hasNextPage\n        }\n      }\n    }\n  }\n": types.CollectionPageDocument,
    "\n  mutation DeleteCollection($collectionId: String!) {\n    deleteCollection(collectionId: $collectionId)\n  }\n": types.DeleteCollectionDocument,
    "\n  query CollectionsIndex {\n    collections {\n      id\n      name\n      description\n      itemCount\n      visibility\n      createdBy {\n        username\n      }\n    }\n  }\n": types.CollectionsIndexDocument,
    "\n  query HomeCollections {\n    home {\n      sections {\n        id\n        ...CollectionShelf\n      }\n    }\n  }\n": types.HomeCollectionsDocument,
    "\n  query GetNodeById($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      kind\n      inWatchlist\n      unavailableAt\n      unplayedCount\n      seasonCount\n      seasonNumber\n      episodeNumber\n      ...GetPathForNode\n      parent {\n        id\n        ...GetPathForNode\n      }\n      root {\n        id\n        ...GetPathForNode\n        properties {\n          displayName\n        }\n      }\n      properties {\n        displayName\n        tagline\n        posterImage {\n          ...ImageAsset\n        }\n        thumbnailImage {\n          ...ImageAsset\n        }\n        logoImage {\n          id\n          signedUrl\n          aspectRatio\n        }\n        backdropImage {\n          ...ImageAsset\n          signedUrl\n          aspectRatio\n        }\n        description\n        contentRating {\n          rating\n        }\n        genres {\n          name\n        }\n        cast {\n          characterName\n          department\n          person {\n            id\n            name\n            profileImage {\n              ...ImageAsset\n            }\n          }\n        }\n      }\n      watchProgressHint\n      currentPlayable {\n        id\n        seasonNumber\n        episodeNumber\n        watchProgressHint\n      }\n      defaultFile {\n        id\n        probe {\n          runtimeMinutes\n          width\n          height\n          videoCodec\n          videoBitrate\n        }\n      }\n      recommendedNodes {\n        id\n        ...NodePoster\n      }\n    }\n  }\n": types.GetNodeByIdDocument,
    "\n  query PlaygroundViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n": types.PlaygroundViewerDocument,
    "\n  query SettingsViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n": types.SettingsViewerDocument,
    "\n  mutation Signup($username: String!, $password: String!, $inviteCode: String) {\n    signup(username: $username, password: $password, inviteCode: $inviteCode) {\n      id\n      username\n    }\n  }\n": types.SignupDocument,
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
export function graphql(source: "\n  query GetActivities {\n    activities {\n      taskType\n      title\n      current\n      total\n      progressPercent\n    }\n  }\n"): (typeof documents)["\n  query GetActivities {\n    activities {\n      taskType\n      title\n      current\n      total\n      progressPercent\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query EditableCollections {\n    collections {\n      id\n      name\n      canEdit\n      resolverKind\n    }\n  }\n"): (typeof documents)["\n  query EditableCollections {\n    collections {\n      id\n      name\n      canEdit\n      resolverKind\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreatePrivateCollection(\n    $name: String!\n    $resolverKind: CollectionResolverKind!\n    $visibility: CollectionVisibility!\n  ) {\n    createCollection(name: $name, resolverKind: $resolverKind, visibility: $visibility) {\n      id\n      name\n    }\n  }\n"): (typeof documents)["\n  mutation CreatePrivateCollection(\n    $name: String!\n    $resolverKind: CollectionResolverKind!\n    $visibility: CollectionVisibility!\n  ) {\n    createCollection(name: $name, resolverKind: $resolverKind, visibility: $visibility) {\n      id\n      name\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation AddNodeToCollection($collectionId: String!, $nodeId: String!) {\n    addNodeToCollection(collectionId: $collectionId, nodeId: $nodeId) {\n      id\n      name\n    }\n  }\n"): (typeof documents)["\n  mutation AddNodeToCollection($collectionId: String!, $nodeId: String!) {\n    addNodeToCollection(collectionId: $collectionId, nodeId: $nodeId) {\n      id\n      name\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment CollectionShelf on Collection {\n    id\n    name\n    itemCount\n    nodeList(first: 12) {\n      nodes {\n        id\n        ...NodePoster\n      }\n      pageInfo {\n        hasNextPage\n      }\n    }\n  }\n"): (typeof documents)["\n  fragment CollectionShelf on Collection {\n    id\n    name\n    itemCount\n    nodeList(first: 12) {\n      nodes {\n        id\n        ...NodePoster\n      }\n      pageInfo {\n        hasNextPage\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  subscription ContentUpdates {\n    contentUpdates\n  }\n"): (typeof documents)["\n  subscription ContentUpdates {\n    contentUpdates\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetFiles($path: String!) {\n    listFiles(path: $path)\n  }\n"): (typeof documents)["\n  query GetFiles($path: String!) {\n    listFiles(path: $path)\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment ImageAsset on Asset {\n    id\n    signedUrl\n    thumbhash\n  }\n"): (typeof documents)["\n  fragment ImageAsset on Asset {\n    id\n    signedUrl\n    thumbhash\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n    importWatchStates(input: $input) {\n      dryRun\n      totalRows\n      matchedRows\n      unmatchedRows\n      conflictRows\n      willInsert\n      willOverwrite\n      imported\n      skipped\n      conflicts {\n        rowIndex\n        sourceItemId\n        title\n        itemId\n        existingProgressPercent\n        importedProgressPercent\n        reason\n      }\n      unmatched {\n        rowIndex\n        sourceItemId\n        title\n        reason\n        ambiguous\n      }\n    }\n  }\n"): (typeof documents)["\n  mutation RunImportWatchStates($input: ImportWatchStatesInput!) {\n    importWatchStates(input: $input) {\n      dryRun\n      totalRows\n      matchedRows\n      unmatchedRows\n      conflictRows\n      willInsert\n      willOverwrite\n      imported\n      skipped\n      conflicts {\n        rowIndex\n        sourceItemId\n        title\n        itemId\n        existingProgressPercent\n        importedProgressPercent\n        reason\n      }\n      unmatched {\n        rowIndex\n        sourceItemId\n        title\n        reason\n        ambiguous\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment EpisodeCard on Node {\n    id\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n      firstAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    watchProgressHint\n    ...GetPathForNode\n  }\n"): (typeof documents)["\n  fragment EpisodeCard on Node {\n    id\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n      firstAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    watchProgressHint\n    ...GetPathForNode\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query NodePage($after: String, $first: Int!, $filter: NodeFilter!) {\n    nodeList(after: $after, first: $first, filter: $filter) {\n      edges {\n        node {\n          id\n          ...NodePoster\n          ...EpisodeCard\n        }\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n"): (typeof documents)["\n  query NodePage($after: String, $first: Int!, $filter: NodeFilter!) {\n    nodeList(after: $after, first: $first, filter: $filter) {\n      edges {\n        node {\n          id\n          ...NodePoster\n          ...EpisodeCard\n        }\n      }\n      pageInfo {\n        endCursor\n        hasNextPage\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment NodePoster on Node {\n    id\n    kind\n    libraryId\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      displayDetail\n      posterImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    seasonCount\n    episodeCount\n    seasonNumber\n    episodeNumber\n    ...GetPathForNode\n  }\n"): (typeof documents)["\n  fragment NodePoster on Node {\n    id\n    kind\n    libraryId\n    inWatchlist\n    unavailableAt\n    properties {\n      displayName\n      displayDetail\n      posterImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    seasonCount\n    episodeCount\n    seasonNumber\n    episodeNumber\n    ...GetPathForNode\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query Player($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      ...GetPathForNode\n      properties {\n        displayName\n        seasonNumber\n        episodeNumber\n      }\n      root {\n        properties {\n          displayName\n        }\n      }\n      defaultFile {\n        id\n        height\n        width\n        resumeHint {\n          startMs\n          updatedAt\n        }\n        probe {\n          durationSeconds\n          width\n          height\n        }\n        playback {\n          hlsUrlTemplate\n          video {\n            ...PlayerVideoTrack\n          }\n          audio {\n            ...PlayerAudioTrack\n          }\n          subtitles {\n            ...PlayerSubtitleTrack\n          }\n        }\n        timelinePreview {\n          ...PlayerTimelinePreviewSheet\n        }\n        segments {\n          kind\n          startMs\n          endMs\n        }\n      }\n      previousPlayable {\n        id\n        ...PlayerItemCard\n      }\n      nextPlayable {\n        id\n        ...PlayerItemCard\n      }\n    }\n  }\n"): (typeof documents)["\n  query Player($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      ...GetPathForNode\n      properties {\n        displayName\n        seasonNumber\n        episodeNumber\n      }\n      root {\n        properties {\n          displayName\n        }\n      }\n      defaultFile {\n        id\n        height\n        width\n        resumeHint {\n          startMs\n          updatedAt\n        }\n        probe {\n          durationSeconds\n          width\n          height\n        }\n        playback {\n          hlsUrlTemplate\n          video {\n            ...PlayerVideoTrack\n          }\n          audio {\n            ...PlayerAudioTrack\n          }\n          subtitles {\n            ...PlayerSubtitleTrack\n          }\n        }\n        timelinePreview {\n          ...PlayerTimelinePreviewSheet\n        }\n        segments {\n          kind\n          startMs\n          endMs\n        }\n      }\n      previousPlayable {\n        id\n        ...PlayerItemCard\n      }\n      nextPlayable {\n        id\n        ...PlayerItemCard\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment PlayerAudioTrack on PlaybackAudioTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    languageBcp47\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n"): (typeof documents)["\n  fragment PlayerAudioTrack on PlaybackAudioTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    languageBcp47\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment PlayerVideoTrack on PlaybackVideoTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n"): (typeof documents)["\n  fragment PlayerVideoTrack on PlaybackVideoTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    renditions {\n      pairId\n      profileId\n      codec\n      displayInfo\n      codecTag\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment PlayerSubtitleTrack on PlaybackSubtitleTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    kind\n    languageBcp47\n    renditions {\n      variantId\n      codec\n      displayInfo\n      signedUrl\n    }\n  }\n"): (typeof documents)["\n  fragment PlayerSubtitleTrack on PlaybackSubtitleTrack {\n    sourceTrackId\n    displayName\n    autoselect\n    kind\n    languageBcp47\n    renditions {\n      variantId\n      codec\n      displayInfo\n      signedUrl\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n    positionMs\n    endMs\n    sheetIntervalMs\n    sheetGapSize\n    asset {\n      id\n      signedUrl\n      width\n      height\n    }\n  }\n"): (typeof documents)["\n  fragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {\n    positionMs\n    endMs\n    sheetIntervalMs\n    sheetGapSize\n    asset {\n      id\n      signedUrl\n      width\n      height\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment PlayerItemCard on Node {\n    id\n    ...GetPathForNode\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n    }\n  }\n"): (typeof documents)["\n  fragment PlayerItemCard on Node {\n    id\n    ...GetPathForNode\n    properties {\n      displayName\n      description\n      thumbnailImage {\n        ...ImageAsset\n      }\n      seasonNumber\n      episodeNumber\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment SearchNodeResult on Node {\n    id\n    kind\n    libraryId\n    root {\n      properties {\n        displayName\n      }\n    }\n    seasonCount\n    episodeCount\n    properties {\n      displayName\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      description\n      seasonNumber\n      episodeNumber\n      firstAired\n      lastAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    ...GetPathForNode\n  }\n"): (typeof documents)["\n  fragment SearchNodeResult on Node {\n    id\n    kind\n    libraryId\n    root {\n      properties {\n        displayName\n      }\n    }\n    seasonCount\n    episodeCount\n    properties {\n      displayName\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      description\n      seasonNumber\n      episodeNumber\n      firstAired\n      lastAired\n    }\n    defaultFile {\n      id\n      probe {\n        runtimeMinutes\n      }\n    }\n    ...GetPathForNode\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SearchMedia($query: String!, $limit: Int!, $kinds: [NodeKind!]!) {\n    nodeList(first: $limit, filter: { searchTerm: $query, kinds: $kinds }) {\n      nodes {\n        ...SearchNodeResult\n      }\n    }\n  }\n"): (typeof documents)["\n  query SearchMedia($query: String!, $limit: Int!, $kinds: [NodeKind!]!) {\n    nodeList(first: $limit, filter: { searchTerm: $query, kinds: $kinds }) {\n      nodes {\n        ...SearchNodeResult\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment SeasonCard on Node {\n    id\n    unavailableAt\n    properties {\n      displayName\n      seasonNumber\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    episodeCount\n    ...GetPathForNode\n  }\n"): (typeof documents)["\n  fragment SeasonCard on Node {\n    id\n    unavailableAt\n    properties {\n      displayName\n      seasonNumber\n      posterImage {\n        ...ImageAsset\n      }\n      thumbnailImage {\n        ...ImageAsset\n      }\n      firstAired\n      lastAired\n    }\n    currentPlayable {\n      id\n      watchProgressHint\n    }\n    unplayedCount\n    episodeCount\n    ...GetPathForNode\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment LibraryCard on Library {\n    id\n    name\n    path\n    pinned\n    createdAt\n    lastScannedAt\n  }\n"): (typeof documents)["\n  fragment LibraryCard on Library {\n    id\n    name\n    path\n    pinned\n    createdAt\n    lastScannedAt\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetLibraries {\n    libraries {\n      id\n      ...LibraryCard\n    }\n  }\n"): (typeof documents)["\n  query GetLibraries {\n    libraries {\n      id\n      ...LibraryCard\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreateLibrary($name: String!, $path: String!, $pinned: Boolean!) {\n    createLibrary(name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n"): (typeof documents)["\n  mutation CreateLibrary($name: String!, $path: String!, $pinned: Boolean!) {\n    createLibrary(name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!, $pinned: Boolean!) {\n    updateLibrary(libraryId: $libraryId, name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateLibrary($libraryId: String!, $name: String!, $path: String!, $pinned: Boolean!) {\n    updateLibrary(libraryId: $libraryId, name: $name, path: $path, pinned: $pinned) {\n      ...LibraryCard\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation DeleteLibrary($libraryId: String!) {\n    deleteLibrary(libraryId: $libraryId)\n  }\n"): (typeof documents)["\n  mutation DeleteLibrary($libraryId: String!) {\n    deleteLibrary(libraryId: $libraryId)\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query UsersManagement {\n    viewer {\n      id\n    }\n    libraries {\n      id\n      name\n      createdAt\n    }\n    users {\n      id\n      ...UserCard\n    }\n  }\n"): (typeof documents)["\n  query UsersManagement {\n    viewer {\n      id\n    }\n    libraries {\n      id\n      name\n      createdAt\n    }\n    users {\n      id\n      ...UserCard\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    createUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n"): (typeof documents)["\n  mutation CreateUserInvite($username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    createUserInvite(username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    updateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateUser($userId: String!, $username: String!, $permissions: Int!, $libraryIds: [String!]!) {\n    updateUser(userId: $userId, username: $username, permissions: $permissions, libraryIds: $libraryIds) {\n      ...UserCard\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation ResetUserInvite($userId: String!) {\n    resetUserInvite(userId: $userId) {\n      ...UserCard\n    }\n  }\n"): (typeof documents)["\n  mutation ResetUserInvite($userId: String!) {\n    resetUserInvite(userId: $userId) {\n      ...UserCard\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation DeleteUser($userId: String!) {\n    deleteUser(userId: $userId)\n  }\n"): (typeof documents)["\n  mutation DeleteUser($userId: String!) {\n    deleteUser(userId: $userId)\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment UserCard on User {\n    id\n    username\n    inviteCode\n    permissions\n    libraries {\n      id\n    }\n    createdAt\n    lastSeenAt\n  }\n"): (typeof documents)["\n  fragment UserCard on User {\n    id\n    username\n    inviteCode\n    permissions\n    libraries {\n      id\n    }\n    createdAt\n    lastSeenAt\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SidebarNavigation {\n    libraries {\n      id\n      name\n      createdAt\n      pinned\n    }\n    collections(pinned: true) {\n      id\n      name\n      createdById\n    }\n  }\n"): (typeof documents)["\n  query SidebarNavigation {\n    libraries {\n      id\n      name\n      createdAt\n      pinned\n    }\n    collections(pinned: true) {\n      id\n      name\n      createdById\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SidebarViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n"): (typeof documents)["\n  query SidebarViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation AddNodeToWatchlist($nodeId: String!) {\n    addNodeToWatchlist(nodeId: $nodeId)\n  }\n"): (typeof documents)["\n  mutation AddNodeToWatchlist($nodeId: String!) {\n    addNodeToWatchlist(nodeId: $nodeId)\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation RemoveNodeFromWatchlist($nodeId: String!) {\n    removeNodeFromWatchlist(nodeId: $nodeId)\n  }\n"): (typeof documents)["\n  mutation RemoveNodeFromWatchlist($nodeId: String!) {\n    removeNodeFromWatchlist(nodeId: $nodeId)\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  fragment GetPathForNode on Node {\n    id\n    libraryId\n  }\n"): (typeof documents)["\n  fragment GetPathForNode on Node {\n    id\n    libraryId\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query CollectionPage($collectionId: String!, $after: String, $first: Int!) {\n    collection(collectionId: $collectionId) {\n      id\n      name\n      description\n      itemCount\n      canDelete\n      nodeList(after: $after, first: $first) {\n        nodes {\n          id\n          ...NodePoster\n        }\n        pageInfo {\n          endCursor\n          hasNextPage\n        }\n      }\n    }\n  }\n"): (typeof documents)["\n  query CollectionPage($collectionId: String!, $after: String, $first: Int!) {\n    collection(collectionId: $collectionId) {\n      id\n      name\n      description\n      itemCount\n      canDelete\n      nodeList(after: $after, first: $first) {\n        nodes {\n          id\n          ...NodePoster\n        }\n        pageInfo {\n          endCursor\n          hasNextPage\n        }\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation DeleteCollection($collectionId: String!) {\n    deleteCollection(collectionId: $collectionId)\n  }\n"): (typeof documents)["\n  mutation DeleteCollection($collectionId: String!) {\n    deleteCollection(collectionId: $collectionId)\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query CollectionsIndex {\n    collections {\n      id\n      name\n      description\n      itemCount\n      visibility\n      createdBy {\n        username\n      }\n    }\n  }\n"): (typeof documents)["\n  query CollectionsIndex {\n    collections {\n      id\n      name\n      description\n      itemCount\n      visibility\n      createdBy {\n        username\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query HomeCollections {\n    home {\n      sections {\n        id\n        ...CollectionShelf\n      }\n    }\n  }\n"): (typeof documents)["\n  query HomeCollections {\n    home {\n      sections {\n        id\n        ...CollectionShelf\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query GetNodeById($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      kind\n      inWatchlist\n      unavailableAt\n      unplayedCount\n      seasonCount\n      seasonNumber\n      episodeNumber\n      ...GetPathForNode\n      parent {\n        id\n        ...GetPathForNode\n      }\n      root {\n        id\n        ...GetPathForNode\n        properties {\n          displayName\n        }\n      }\n      properties {\n        displayName\n        tagline\n        posterImage {\n          ...ImageAsset\n        }\n        thumbnailImage {\n          ...ImageAsset\n        }\n        logoImage {\n          id\n          signedUrl\n          aspectRatio\n        }\n        backdropImage {\n          ...ImageAsset\n          signedUrl\n          aspectRatio\n        }\n        description\n        contentRating {\n          rating\n        }\n        genres {\n          name\n        }\n        cast {\n          characterName\n          department\n          person {\n            id\n            name\n            profileImage {\n              ...ImageAsset\n            }\n          }\n        }\n      }\n      watchProgressHint\n      currentPlayable {\n        id\n        seasonNumber\n        episodeNumber\n        watchProgressHint\n      }\n      defaultFile {\n        id\n        probe {\n          runtimeMinutes\n          width\n          height\n          videoCodec\n          videoBitrate\n        }\n      }\n      recommendedNodes {\n        id\n        ...NodePoster\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetNodeById($nodeId: String!) {\n    node(nodeId: $nodeId) {\n      id\n      kind\n      inWatchlist\n      unavailableAt\n      unplayedCount\n      seasonCount\n      seasonNumber\n      episodeNumber\n      ...GetPathForNode\n      parent {\n        id\n        ...GetPathForNode\n      }\n      root {\n        id\n        ...GetPathForNode\n        properties {\n          displayName\n        }\n      }\n      properties {\n        displayName\n        tagline\n        posterImage {\n          ...ImageAsset\n        }\n        thumbnailImage {\n          ...ImageAsset\n        }\n        logoImage {\n          id\n          signedUrl\n          aspectRatio\n        }\n        backdropImage {\n          ...ImageAsset\n          signedUrl\n          aspectRatio\n        }\n        description\n        contentRating {\n          rating\n        }\n        genres {\n          name\n        }\n        cast {\n          characterName\n          department\n          person {\n            id\n            name\n            profileImage {\n              ...ImageAsset\n            }\n          }\n        }\n      }\n      watchProgressHint\n      currentPlayable {\n        id\n        seasonNumber\n        episodeNumber\n        watchProgressHint\n      }\n      defaultFile {\n        id\n        probe {\n          runtimeMinutes\n          width\n          height\n          videoCodec\n          videoBitrate\n        }\n      }\n      recommendedNodes {\n        id\n        ...NodePoster\n      }\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query PlaygroundViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n"): (typeof documents)["\n  query PlaygroundViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  query SettingsViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n"): (typeof documents)["\n  query SettingsViewer {\n    viewer {\n      id\n      permissions\n    }\n  }\n"];
/**
 * The graphql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function graphql(source: "\n  mutation Signup($username: String!, $password: String!, $inviteCode: String) {\n    signup(username: $username, password: $password, inviteCode: $inviteCode) {\n      id\n      username\n    }\n  }\n"): (typeof documents)["\n  mutation Signup($username: String!, $password: String!, $inviteCode: String) {\n    signup(username: $username, password: $password, inviteCode: $inviteCode) {\n      id\n      username\n    }\n  }\n"];

export function graphql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;