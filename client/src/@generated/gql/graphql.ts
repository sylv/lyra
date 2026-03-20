/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = T | null | undefined;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
};

export type Activity = {
  __typename: 'Activity';
  current: Scalars['Int']['output'];
  progressPercent: Scalars['Float']['output'];
  taskType: Scalars['String']['output'];
  title: Scalars['String']['output'];
  total: Scalars['Int']['output'];
};

export type Asset = {
  __typename: 'Asset';
  createdAt: Scalars['Int']['output'];
  hashSha256: Maybe<Scalars['String']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  id: Scalars['String']['output'];
  mimeType: Maybe<Scalars['String']['output']>;
  signedUrl: Scalars['String']['output'];
  sizeBytes: Maybe<Scalars['Int']['output']>;
  sourceUrl: Maybe<Scalars['String']['output']>;
  thumbhash: Maybe<Scalars['String']['output']>;
  width: Maybe<Scalars['Int']['output']>;
};

export enum ContentUpdateEvent {
  ContentUpdate = 'CONTENT_UPDATE'
}

export type File = {
  __typename: 'File';
  discoveredAt: Scalars['Int']['output'];
  editionName: Maybe<Scalars['String']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  id: Scalars['String']['output'];
  libraryId: Scalars['String']['output'];
  relativePath: Scalars['String']['output'];
  scannedAt: Maybe<Scalars['Int']['output']>;
  segments: Array<FileSegment>;
  sizeBytes: Scalars['Int']['output'];
  timelinePreview: Array<TimelinePreviewSheet>;
  unavailableAt: Maybe<Scalars['Int']['output']>;
  width: Maybe<Scalars['Int']['output']>;
};

export type FileSegment = {
  __typename: 'FileSegment';
  endMs: Scalars['Int']['output'];
  kind: FileSegmentKind;
  startMs: Scalars['Int']['output'];
};

export enum FileSegmentKind {
  Intro = 'INTRO'
}

export type ImportWatchStateConflict = {
  __typename: 'ImportWatchStateConflict';
  existingProgressPercent: Scalars['Float']['output'];
  importedProgressPercent: Scalars['Float']['output'];
  itemId: Scalars['String']['output'];
  reason: Scalars['String']['output'];
  rowIndex: Scalars['Int']['output'];
  sourceItemId: Maybe<Scalars['String']['output']>;
  title: Maybe<Scalars['String']['output']>;
};

export type ImportWatchStateRowInput = {
  episodeNumber?: InputMaybe<Scalars['Int']['input']>;
  fileBasename?: InputMaybe<Scalars['String']['input']>;
  filePath?: InputMaybe<Scalars['String']['input']>;
  fileSizeBytes?: InputMaybe<Scalars['Int']['input']>;
  imdbId?: InputMaybe<Scalars['String']['input']>;
  mediaType?: InputMaybe<Scalars['String']['input']>;
  progressPercent: Scalars['Float']['input'];
  seasonNumber?: InputMaybe<Scalars['Int']['input']>;
  source: Scalars['String']['input'];
  sourceItemId?: InputMaybe<Scalars['String']['input']>;
  title?: InputMaybe<Scalars['String']['input']>;
  tmdbId?: InputMaybe<Scalars['Int']['input']>;
  viewedAt?: InputMaybe<Scalars['Int']['input']>;
};

export type ImportWatchStateUnmatched = {
  __typename: 'ImportWatchStateUnmatched';
  ambiguous: Scalars['Boolean']['output'];
  reason: Scalars['String']['output'];
  rowIndex: Scalars['Int']['output'];
  sourceItemId: Maybe<Scalars['String']['output']>;
  title: Maybe<Scalars['String']['output']>;
};

export type ImportWatchStatesInput = {
  dryRun: Scalars['Boolean']['input'];
  overwriteConflicts: Scalars['Boolean']['input'];
  rows: Array<ImportWatchStateRowInput>;
};

export type ImportWatchStatesResult = {
  __typename: 'ImportWatchStatesResult';
  conflictRows: Scalars['Int']['output'];
  conflicts: Array<ImportWatchStateConflict>;
  dryRun: Scalars['Boolean']['output'];
  imported: Scalars['Int']['output'];
  matchedRows: Scalars['Int']['output'];
  skipped: Scalars['Int']['output'];
  totalRows: Scalars['Int']['output'];
  unmatched: Array<ImportWatchStateUnmatched>;
  unmatchedRows: Scalars['Int']['output'];
  willInsert: Scalars['Int']['output'];
  willOverwrite: Scalars['Int']['output'];
};

export type Library = {
  __typename: 'Library';
  createdAt: Scalars['Int']['output'];
  id: Scalars['String']['output'];
  lastScannedAt: Maybe<Scalars['Int']['output']>;
  name: Scalars['String']['output'];
  path: Scalars['String']['output'];
};

export type Mutation = {
  __typename: 'Mutation';
  createLibrary: Library;
  createUserInvite: User;
  deleteLibrary: Scalars['Boolean']['output'];
  deleteUser: Scalars['Boolean']['output'];
  importWatchStates: ImportWatchStatesResult;
  resetUserInvite: User;
  signup: User;
  updateLibrary: Library;
  updateUser: User;
  updateWatchProgress: Array<WatchProgress>;
};


export type MutationCreateLibraryArgs = {
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
};


export type MutationCreateUserInviteArgs = {
  permissions: Scalars['Int']['input'];
  username: Scalars['String']['input'];
};


export type MutationDeleteLibraryArgs = {
  libraryId: Scalars['String']['input'];
};


export type MutationDeleteUserArgs = {
  userId: Scalars['String']['input'];
};


export type MutationImportWatchStatesArgs = {
  input: ImportWatchStatesInput;
};


export type MutationResetUserInviteArgs = {
  userId: Scalars['String']['input'];
};


export type MutationSignupArgs = {
  inviteCode?: InputMaybe<Scalars['String']['input']>;
  password: Scalars['String']['input'];
  permissions?: InputMaybe<Scalars['Int']['input']>;
  username: Scalars['String']['input'];
};


export type MutationUpdateLibraryArgs = {
  libraryId: Scalars['String']['input'];
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
};


export type MutationUpdateUserArgs = {
  permissions: Scalars['Int']['input'];
  userId: Scalars['String']['input'];
  username: Scalars['String']['input'];
};


export type MutationUpdateWatchProgressArgs = {
  fileId: Scalars['String']['input'];
  progressPercent: Scalars['Float']['input'];
  userId?: InputMaybe<Scalars['String']['input']>;
};

export type Node = {
  __typename: 'Node';
  children: Array<Node>;
  createdAt: Scalars['Int']['output'];
  episodeCount: Scalars['Int']['output'];
  episodeNumber: Maybe<Scalars['Int']['output']>;
  file: Maybe<File>;
  id: Scalars['String']['output'];
  kind: NodeKind;
  lastAddedAt: Scalars['Int']['output'];
  libraryId: Scalars['String']['output'];
  name: Scalars['String']['output'];
  nextPlayable: Maybe<Node>;
  order: Scalars['Int']['output'];
  parent: Maybe<Node>;
  parentId: Maybe<Scalars['String']['output']>;
  previousPlayable: Maybe<Node>;
  properties: NodeProperties;
  root: Maybe<Node>;
  rootId: Scalars['String']['output'];
  seasonCount: Scalars['Int']['output'];
  seasonNumber: Maybe<Scalars['Int']['output']>;
  unplayedCount: Scalars['Int']['output'];
  updatedAt: Scalars['Int']['output'];
  watchProgress: Maybe<WatchProgress>;
};

export type NodeConnection = {
  __typename: 'NodeConnection';
  /** A list of edges. */
  edges: Array<NodeEdge>;
  /** A list of nodes. */
  nodes: Array<Node>;
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
};

/** An edge in a connection. */
export type NodeEdge = {
  __typename: 'NodeEdge';
  /** A cursor for use in pagination */
  cursor: Scalars['String']['output'];
  /** The item at the end of the edge */
  node: Node;
};

export type NodeFilter = {
  kinds?: InputMaybe<Array<NodeKind>>;
  libraryId?: InputMaybe<Scalars['String']['input']>;
  orderBy?: InputMaybe<OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  parentId?: InputMaybe<Scalars['String']['input']>;
  rootId?: InputMaybe<Scalars['String']['input']>;
  watched?: InputMaybe<Scalars['Boolean']['input']>;
};

export enum NodeKind {
  Episode = 'EPISODE',
  Movie = 'MOVIE',
  Season = 'SEASON',
  Series = 'SERIES'
}

export type NodeProperties = {
  __typename: 'NodeProperties';
  audioBitrate: Maybe<Scalars['Int']['output']>;
  audioChannels: Maybe<Scalars['Int']['output']>;
  audioCodec: Maybe<Scalars['String']['output']>;
  backgroundImage: Maybe<Asset>;
  createdAt: Maybe<Scalars['Int']['output']>;
  description: Maybe<Scalars['String']['output']>;
  durationSeconds: Maybe<Scalars['Int']['output']>;
  endedAt: Maybe<Scalars['Int']['output']>;
  episodeNumber: Maybe<Scalars['Int']['output']>;
  fileSizeBytes: Maybe<Scalars['Int']['output']>;
  fps: Maybe<Scalars['Float']['output']>;
  hasSubtitles: Maybe<Scalars['Boolean']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  posterImage: Maybe<Asset>;
  rating: Maybe<Scalars['Float']['output']>;
  releasedAt: Maybe<Scalars['Int']['output']>;
  runtimeMinutes: Maybe<Scalars['Int']['output']>;
  seasonNumber: Maybe<Scalars['Int']['output']>;
  thumbnailImage: Maybe<Asset>;
  updatedAt: Maybe<Scalars['Int']['output']>;
  videoBitrate: Maybe<Scalars['Int']['output']>;
  videoCodec: Maybe<Scalars['String']['output']>;
  width: Maybe<Scalars['Int']['output']>;
};

export enum OrderBy {
  AddedAt = 'ADDED_AT',
  Alphabetical = 'ALPHABETICAL',
  Order = 'ORDER',
  Rating = 'RATING',
  ReleasedAt = 'RELEASED_AT'
}

export enum OrderDirection {
  Asc = 'ASC',
  Desc = 'DESC'
}

/** Information about pagination in a connection */
export type PageInfo = {
  __typename: 'PageInfo';
  /** When paginating forwards, the cursor to continue. */
  endCursor: Maybe<Scalars['String']['output']>;
  /** When paginating forwards, are there more items? */
  hasNextPage: Scalars['Boolean']['output'];
  /** When paginating backwards, are there more items? */
  hasPreviousPage: Scalars['Boolean']['output'];
  /** When paginating backwards, the cursor to continue. */
  startCursor: Maybe<Scalars['String']['output']>;
};

export type Query = {
  __typename: 'Query';
  activities: Array<Activity>;
  libraries: Array<Library>;
  library: Library;
  listFiles: Array<Scalars['String']['output']>;
  node: Node;
  nodeList: NodeConnection;
  search: SearchResults;
  users: Array<User>;
  viewer: Maybe<User>;
};


export type QueryLibraryArgs = {
  libraryId: Scalars['String']['input'];
};


export type QueryListFilesArgs = {
  path: Scalars['String']['input'];
};


export type QueryNodeArgs = {
  nodeId: Scalars['String']['input'];
};


export type QueryNodeListArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  filter: NodeFilter;
  first?: InputMaybe<Scalars['Int']['input']>;
};


export type QuerySearchArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  query: Scalars['String']['input'];
};

export type SearchResults = {
  __typename: 'SearchResults';
  episodes: Array<Node>;
  roots: Array<Node>;
};

export type SubscriptionRoot = {
  __typename: 'SubscriptionRoot';
  contentUpdates: ContentUpdateEvent;
};

export type TimelinePreviewSheet = {
  __typename: 'TimelinePreviewSheet';
  asset: Asset;
  endMs: Scalars['Int']['output'];
  positionMs: Scalars['Int']['output'];
  sheetGapSize: Scalars['Int']['output'];
  sheetIntervalMs: Scalars['Int']['output'];
};

export type User = {
  __typename: 'User';
  createdAt: Scalars['Int']['output'];
  id: Scalars['String']['output'];
  inviteCode: Maybe<Scalars['String']['output']>;
  lastSeenAt: Maybe<Scalars['Int']['output']>;
  permissions: Scalars['Int']['output'];
  username: Scalars['String']['output'];
};

export type WatchProgress = {
  __typename: 'WatchProgress';
  completed: Scalars['Boolean']['output'];
  createdAt: Scalars['Int']['output'];
  fileId: Scalars['String']['output'];
  id: Scalars['String']['output'];
  nodeId: Scalars['String']['output'];
  progressPercent: Scalars['Float']['output'];
  updatedAt: Scalars['Int']['output'];
  userId: Scalars['String']['output'];
};

export type GetActivitiesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetActivitiesQuery = { activities: Array<{ __typename: 'Activity', taskType: string, title: string, current: number, total: number, progressPercent: number }> };

export type ContentUpdatesSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type ContentUpdatesSubscription = { contentUpdates: ContentUpdateEvent };

export type GetFilesQueryVariables = Exact<{
  path: Scalars['String']['input'];
}>;


export type GetFilesQuery = { listFiles: Array<string> };

export type EpisodeCardFragment = (
  { __typename: 'Node', id: string, name: string, properties: { __typename: 'NodeProperties', description: string | null, seasonNumber: number | null, episodeNumber: number | null, releasedAt: number | null, runtimeMinutes: number | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'EpisodeCardFragment' };

export type ImageAssetFragment = { __typename: 'Asset', id: string, signedUrl: string, thumbhash: string | null } & { ' $fragmentName'?: 'ImageAssetFragment' };

export type RunImportWatchStatesMutationVariables = Exact<{
  input: ImportWatchStatesInput;
}>;


export type RunImportWatchStatesMutation = { importWatchStates: { __typename: 'ImportWatchStatesResult', dryRun: boolean, totalRows: number, matchedRows: number, unmatchedRows: number, conflictRows: number, willInsert: number, willOverwrite: number, imported: number, skipped: number, conflicts: Array<{ __typename: 'ImportWatchStateConflict', rowIndex: number, sourceItemId: string | null, title: string | null, itemId: string, existingProgressPercent: number, importedProgressPercent: number, reason: string }>, unmatched: Array<{ __typename: 'ImportWatchStateUnmatched', rowIndex: number, sourceItemId: string | null, title: string | null, reason: string, ambiguous: boolean }> } };

export type NodeListFragment = (
  { __typename: 'Node', id: string }
  & { ' $fragmentRefs'?: { 'NodePosterFragment': NodePosterFragment } }
) & { ' $fragmentName'?: 'NodeListFragment' };

export type NodePosterFragment = (
  { __typename: 'Node', id: string, name: string, kind: NodeKind, libraryId: string, unplayedCount: number, seasonCount: number, episodeCount: number, properties: { __typename: 'NodeProperties', releasedAt: number | null, endedAt: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, nextPlayable: { __typename: 'Node', id: string, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'NodePosterFragment' };

export type PlayerTimelinePreviewSheetFragment = { __typename: 'TimelinePreviewSheet', positionMs: number, endMs: number, sheetIntervalMs: number, sheetGapSize: number, asset: { __typename: 'Asset', id: string, signedUrl: string, width: number | null, height: number | null } } & { ' $fragmentName'?: 'PlayerTimelinePreviewSheetFragment' };

export type UpdateWatchStateMutationVariables = Exact<{
  fileId: Scalars['String']['input'];
  progressPercent: Scalars['Float']['input'];
}>;


export type UpdateWatchStateMutation = { updateWatchProgress: Array<{ __typename: 'WatchProgress', progressPercent: number, updatedAt: number }> };

export type ItemPlaybackQueryVariables = Exact<{
  itemId: Scalars['String']['input'];
}>;


export type ItemPlaybackQuery = { node: { __typename: 'Node', id: string, libraryId: string, kind: NodeKind, name: string, properties: { __typename: 'NodeProperties', seasonNumber: number | null, episodeNumber: number | null, runtimeMinutes: number | null }, root: { __typename: 'Node', name: string, libraryId: string } | null, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null, file: { __typename: 'File', id: string, segments: Array<{ __typename: 'FileSegment', kind: FileSegmentKind, startMs: number, endMs: number }>, timelinePreview: Array<(
        { __typename: 'TimelinePreviewSheet' }
        & { ' $fragmentRefs'?: { 'PlayerTimelinePreviewSheetFragment': PlayerTimelinePreviewSheetFragment } }
      )> } | null, previousPlayable: { __typename: 'Node', id: string } | null, nextPlayable: { __typename: 'Node', id: string } | null } };

export type SearchNodeResultFragment = (
  { __typename: 'Node', id: string, name: string, kind: NodeKind, libraryId: string, seasonCount: number, episodeCount: number, root: { __typename: 'Node', name: string } | null, properties: { __typename: 'NodeProperties', description: string | null, seasonNumber: number | null, episodeNumber: number | null, releasedAt: number | null, endedAt: number | null, runtimeMinutes: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null } }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'SearchNodeResultFragment' };

export type SearchMediaQueryVariables = Exact<{
  query: Scalars['String']['input'];
  limit?: InputMaybe<Scalars['Int']['input']>;
}>;


export type SearchMediaQuery = { search: { __typename: 'SearchResults', roots: Array<(
      { __typename: 'Node' }
      & { ' $fragmentRefs'?: { 'SearchNodeResultFragment': SearchNodeResultFragment } }
    )>, episodes: Array<(
      { __typename: 'Node' }
      & { ' $fragmentRefs'?: { 'SearchNodeResultFragment': SearchNodeResultFragment } }
    )> } };

export type SeasonCardFragment = (
  { __typename: 'Node', id: string, name: string, unplayedCount: number, episodeCount: number, properties: { __typename: 'NodeProperties', seasonNumber: number | null, releasedAt: number | null, endedAt: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, nextPlayable: { __typename: 'Node', id: string, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'SeasonCardFragment' };

export type LibraryCardFragment = { __typename: 'Library', id: string, name: string, path: string, createdAt: number, lastScannedAt: number | null } & { ' $fragmentName'?: 'LibraryCardFragment' };

export type GetLibrariesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetLibrariesQuery = { libraries: Array<(
    { __typename: 'Library', id: string }
    & { ' $fragmentRefs'?: { 'LibraryCardFragment': LibraryCardFragment } }
  )> };

export type CreateLibraryMutationVariables = Exact<{
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
}>;


export type CreateLibraryMutation = { createLibrary: (
    { __typename: 'Library' }
    & { ' $fragmentRefs'?: { 'LibraryCardFragment': LibraryCardFragment } }
  ) };

export type UpdateLibraryMutationVariables = Exact<{
  libraryId: Scalars['String']['input'];
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
}>;


export type UpdateLibraryMutation = { updateLibrary: (
    { __typename: 'Library' }
    & { ' $fragmentRefs'?: { 'LibraryCardFragment': LibraryCardFragment } }
  ) };

export type DeleteLibraryMutationVariables = Exact<{
  libraryId: Scalars['String']['input'];
}>;


export type DeleteLibraryMutation = { deleteLibrary: boolean };

export type UsersManagementQueryVariables = Exact<{ [key: string]: never; }>;


export type UsersManagementQuery = { viewer: { __typename: 'User', id: string } | null, users: Array<(
    { __typename: 'User', id: string }
    & { ' $fragmentRefs'?: { 'UserCardFragment': UserCardFragment } }
  )> };

export type CreateUserInviteMutationVariables = Exact<{
  username: Scalars['String']['input'];
  permissions: Scalars['Int']['input'];
}>;


export type CreateUserInviteMutation = { createUserInvite: (
    { __typename: 'User' }
    & { ' $fragmentRefs'?: { 'UserCardFragment': UserCardFragment } }
  ) };

export type UpdateUserMutationVariables = Exact<{
  userId: Scalars['String']['input'];
  username: Scalars['String']['input'];
  permissions: Scalars['Int']['input'];
}>;


export type UpdateUserMutation = { updateUser: (
    { __typename: 'User' }
    & { ' $fragmentRefs'?: { 'UserCardFragment': UserCardFragment } }
  ) };

export type ResetUserInviteMutationVariables = Exact<{
  userId: Scalars['String']['input'];
}>;


export type ResetUserInviteMutation = { resetUserInvite: (
    { __typename: 'User' }
    & { ' $fragmentRefs'?: { 'UserCardFragment': UserCardFragment } }
  ) };

export type DeleteUserMutationVariables = Exact<{
  userId: Scalars['String']['input'];
}>;


export type DeleteUserMutation = { deleteUser: boolean };

export type UserCardFragment = { __typename: 'User', id: string, username: string, inviteCode: string | null, permissions: number, createdAt: number, lastSeenAt: number | null } & { ' $fragmentName'?: 'UserCardFragment' };

export type LibrariesQueryVariables = Exact<{ [key: string]: never; }>;


export type LibrariesQuery = { libraries: Array<{ __typename: 'Library', id: string, name: string, createdAt: number }> };

export type GetPathForNodeFragment = { __typename: 'Node', id: string, libraryId: string } & { ' $fragmentName'?: 'GetPathForNodeFragment' };

export type GetAllNodesQueryVariables = Exact<{
  filter: NodeFilter;
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetAllNodesQuery = { nodeList: { __typename: 'NodeConnection', edges: Array<{ __typename: 'NodeEdge', node: (
        { __typename: 'Node' }
        & { ' $fragmentRefs'?: { 'NodeListFragment': NodeListFragment } }
      ) }>, pageInfo: { __typename: 'PageInfo', endCursor: string | null, hasNextPage: boolean } } };

export type GetLibraryNodesQueryVariables = Exact<{
  libraryId: Scalars['String']['input'];
  filter: NodeFilter;
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetLibraryNodesQuery = { nodeList: { __typename: 'NodeConnection', edges: Array<{ __typename: 'NodeEdge', node: (
        { __typename: 'Node', id: string }
        & { ' $fragmentRefs'?: { 'NodeListFragment': NodeListFragment } }
      ) }>, pageInfo: { __typename: 'PageInfo', endCursor: string | null, hasNextPage: boolean } }, library: { __typename: 'Library', id: string, name: string } };

export type GetNodeByIdQueryVariables = Exact<{
  nodeId: Scalars['String']['input'];
}>;


export type GetNodeByIdQuery = { node: { __typename: 'Node', id: string, libraryId: string, kind: NodeKind, name: string, seasonNumber: number | null, episodeNumber: number | null, unplayedCount: number, parent: { __typename: 'Node', id: string, name: string, libraryId: string } | null, root: { __typename: 'Node', id: string, name: string } | null, children: Array<(
      { __typename: 'Node', id: string, kind: NodeKind, order: number }
      & { ' $fragmentRefs'?: { 'SeasonCardFragment': SeasonCardFragment;'EpisodeCardFragment': EpisodeCardFragment } }
    )>, properties: { __typename: 'NodeProperties', releasedAt: number | null, endedAt: number | null, runtimeMinutes: number | null, description: string | null, posterImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, backgroundImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, thumbnailImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null }, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null, nextPlayable: { __typename: 'Node', id: string, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null } | null, previousPlayable: { __typename: 'Node', id: string } | null } };

export type PlaygroundViewerQueryVariables = Exact<{ [key: string]: never; }>;


export type PlaygroundViewerQuery = { viewer: { __typename: 'User', id: string, permissions: number } | null };

export type SettingsViewerQueryVariables = Exact<{ [key: string]: never; }>;


export type SettingsViewerQuery = { viewer: { __typename: 'User', id: string, permissions: number } | null };

export type SignupMutationVariables = Exact<{
  username: Scalars['String']['input'];
  password: Scalars['String']['input'];
  inviteCode?: InputMaybe<Scalars['String']['input']>;
}>;


export type SignupMutation = { signup: { __typename: 'User', id: string, username: string } };

export const ImageAssetFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}}]} as unknown as DocumentNode<ImageAssetFragment, unknown>;
export const GetPathForNodeFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<GetPathForNodeFragment, unknown>;
export const EpisodeCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<EpisodeCardFragment, unknown>;
export const NodePosterFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<NodePosterFragment, unknown>;
export const NodeListFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodeList"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<NodeListFragment, unknown>;
export const PlayerTimelinePreviewSheetFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}}]} as unknown as DocumentNode<PlayerTimelinePreviewSheetFragment, unknown>;
export const SearchNodeResultFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchNodeResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<SearchNodeResultFragment, unknown>;
export const SeasonCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SeasonCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<SeasonCardFragment, unknown>;
export const LibraryCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<LibraryCardFragment, unknown>;
export const UserCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UserCardFragment, unknown>;
export const GetActivitiesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetActivities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskType"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"current"}},{"kind":"Field","name":{"kind":"Name","value":"total"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}}]}}]}}]} as unknown as DocumentNode<GetActivitiesQuery, GetActivitiesQueryVariables>;
export const ContentUpdatesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"ContentUpdates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"contentUpdates"}}]}}]} as unknown as DocumentNode<ContentUpdatesSubscription, ContentUpdatesSubscriptionVariables>;
export const GetFilesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetFiles"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"listFiles"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}]}]}}]} as unknown as DocumentNode<GetFilesQuery, GetFilesQueryVariables>;
export const RunImportWatchStatesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"RunImportWatchStates"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ImportWatchStatesInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"importWatchStates"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"totalRows"}},{"kind":"Field","name":{"kind":"Name","value":"matchedRows"}},{"kind":"Field","name":{"kind":"Name","value":"unmatchedRows"}},{"kind":"Field","name":{"kind":"Name","value":"conflictRows"}},{"kind":"Field","name":{"kind":"Name","value":"willInsert"}},{"kind":"Field","name":{"kind":"Name","value":"willOverwrite"}},{"kind":"Field","name":{"kind":"Name","value":"imported"}},{"kind":"Field","name":{"kind":"Name","value":"skipped"}},{"kind":"Field","name":{"kind":"Name","value":"conflicts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rowIndex"}},{"kind":"Field","name":{"kind":"Name","value":"sourceItemId"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"itemId"}},{"kind":"Field","name":{"kind":"Name","value":"existingProgressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"importedProgressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"reason"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unmatched"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rowIndex"}},{"kind":"Field","name":{"kind":"Name","value":"sourceItemId"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"reason"}},{"kind":"Field","name":{"kind":"Name","value":"ambiguous"}}]}}]}}]}}]} as unknown as DocumentNode<RunImportWatchStatesMutation, RunImportWatchStatesMutationVariables>;
export const UpdateWatchStateDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateWatchState"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"fileId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"progressPercent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Float"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateWatchProgress"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"fileId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"fileId"}}},{"kind":"Argument","name":{"kind":"Name","value":"progressPercent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"progressPercent"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}}]} as unknown as DocumentNode<UpdateWatchStateMutation, UpdateWatchStateMutationVariables>;
export const ItemPlaybackDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ItemPlayback"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"nodeId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"file"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"segments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"startMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}}]}},{"kind":"Field","name":{"kind":"Name","value":"timelinePreview"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"previousPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}}]} as unknown as DocumentNode<ItemPlaybackQuery, ItemPlaybackQueryVariables>;
export const SearchMediaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SearchMedia"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"query"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"limit"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"search"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"query"},"value":{"kind":"Variable","name":{"kind":"Name","value":"query"}}},{"kind":"Argument","name":{"kind":"Name","value":"limit"},"value":{"kind":"Variable","name":{"kind":"Name","value":"limit"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"roots"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SearchNodeResult"}}]}},{"kind":"Field","name":{"kind":"Name","value":"episodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SearchNodeResult"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchNodeResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<SearchMediaQuery, SearchMediaQueryVariables>;
export const GetLibrariesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLibraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<GetLibrariesQuery, GetLibrariesQueryVariables>;
export const CreateLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<CreateLibraryMutation, CreateLibraryMutationVariables>;
export const UpdateLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}},{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<UpdateLibraryMutation, UpdateLibraryMutationVariables>;
export const DeleteLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}}]}]}}]} as unknown as DocumentNode<DeleteLibraryMutation, DeleteLibraryMutationVariables>;
export const UsersManagementDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"UsersManagement"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"users"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UsersManagementQuery, UsersManagementQueryVariables>;
export const CreateUserInviteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateUserInvite"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createUserInvite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"permissions"},"value":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<CreateUserInviteMutation, CreateUserInviteMutationVariables>;
export const UpdateUserDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateUser"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateUser"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}},{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"permissions"},"value":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UpdateUserMutation, UpdateUserMutationVariables>;
export const ResetUserInviteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ResetUserInvite"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"resetUserInvite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<ResetUserInviteMutation, ResetUserInviteMutationVariables>;
export const DeleteUserDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteUser"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteUser"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}}]}]}}]} as unknown as DocumentNode<DeleteUserMutation, DeleteUserMutationVariables>;
export const LibrariesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]} as unknown as DocumentNode<LibrariesQuery, LibrariesQueryVariables>;
export const GetAllNodesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetAllNodes"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"NodeFilter"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}},{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"45"}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodeList"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodeList"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}}]} as unknown as DocumentNode<GetAllNodesQuery, GetAllNodesQueryVariables>;
export const GetLibraryNodesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLibraryNodes"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"NodeFilter"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}},{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"45"}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodeList"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"library"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodeList"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}}]} as unknown as DocumentNode<GetLibraryNodesQuery, GetLibraryNodesQueryVariables>;
export const GetNodeByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetNodeById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"nodeId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}},{"kind":"Field","name":{"kind":"Name","value":"children"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"order"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"SeasonCard"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"EpisodeCard"}}]}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"backgroundImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"previousPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SeasonCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<GetNodeByIdQuery, GetNodeByIdQueryVariables>;
export const PlaygroundViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PlaygroundViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<PlaygroundViewerQuery, PlaygroundViewerQueryVariables>;
export const SettingsViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SettingsViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<SettingsViewerQuery, SettingsViewerQueryVariables>;
export const SignupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Signup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"password"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"inviteCode"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"signup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"password"},"value":{"kind":"Variable","name":{"kind":"Name","value":"password"}}},{"kind":"Argument","name":{"kind":"Name","value":"inviteCode"},"value":{"kind":"Variable","name":{"kind":"Name","value":"inviteCode"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}}]}}]} as unknown as DocumentNode<SignupMutation, SignupMutationVariables>;