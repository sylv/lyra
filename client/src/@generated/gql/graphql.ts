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
  current: Maybe<Scalars['Int']['output']>;
  progressPercent: Maybe<Scalars['Float']['output']>;
  taskType: Scalars['String']['output'];
  title: Scalars['String']['output'];
  total: Maybe<Scalars['Int']['output']>;
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

export type Collection = {
  __typename: 'Collection';
  canDelete: Scalars['Boolean']['output'];
  canEdit: Scalars['Boolean']['output'];
  createdAt: Scalars['Int']['output'];
  createdBy: Maybe<User>;
  createdById: Maybe<Scalars['String']['output']>;
  description: Maybe<Scalars['String']['output']>;
  homePosition: Scalars['Int']['output'];
  id: Scalars['String']['output'];
  itemCount: Scalars['Int']['output'];
  kind: Maybe<CollectionKind>;
  name: Scalars['String']['output'];
  nodeList: NodeConnection;
  pinned: Scalars['Boolean']['output'];
  pinnedPosition: Scalars['Int']['output'];
  resolverKind: CollectionResolverKind;
  showOnHome: Scalars['Boolean']['output'];
  updatedAt: Scalars['Int']['output'];
  visibility: CollectionVisibility;
};


export type CollectionNodeListArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  first?: InputMaybe<Scalars['Int']['input']>;
};

export enum CollectionKind {
  ContinueWatching = 'CONTINUE_WATCHING'
}

export enum CollectionResolverKind {
  Filter = 'FILTER',
  Manual = 'MANUAL'
}

export enum CollectionVisibility {
  Private = 'PRIVATE',
  Public = 'PUBLIC'
}

export enum ContentUpdateEvent {
  ContentUpdate = 'CONTENT_UPDATE'
}

export enum EffectiveWatchSessionState {
  Buffering = 'BUFFERING',
  InactivePlayers = 'INACTIVE_PLAYERS',
  Paused = 'PAUSED',
  Playing = 'PLAYING'
}

export type File = {
  __typename: 'File';
  discoveredAt: Scalars['Int']['output'];
  editionName: Maybe<Scalars['String']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  id: Scalars['String']['output'];
  libraryId: Scalars['String']['output'];
  recommendedTracks: Array<RecommendedTrack>;
  relativePath: Scalars['String']['output'];
  scannedAt: Maybe<Scalars['Int']['output']>;
  segments: Array<FileSegment>;
  sizeBytes: Scalars['Int']['output'];
  timelinePreview: Array<TimelinePreviewSheet>;
  tracks: Array<TrackInfo>;
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

export type HomeView = {
  __typename: 'HomeView';
  sections: Array<Collection>;
};

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
  pinned: Scalars['Boolean']['output'];
  unavailableAt: Maybe<Scalars['Int']['output']>;
};

export type Mutation = {
  __typename: 'Mutation';
  addNodeToCollection: Collection;
  createCollection: Collection;
  createLibrary: Library;
  createUserInvite: User;
  deleteCollection: Scalars['Boolean']['output'];
  deleteLibrary: Scalars['Boolean']['output'];
  deleteUser: Scalars['Boolean']['output'];
  importWatchStates: ImportWatchStatesResult;
  leaveWatchSession: Scalars['Boolean']['output'];
  resetUserInvite: User;
  setPreferredAudio: User;
  setPreferredSubtitle: User;
  signup: User;
  updateCollection: Collection;
  updateLibrary: Library;
  updateUser: User;
  updateWatchProgress: Array<WatchProgress>;
  watchSessionAction: WatchSessionBeacon;
  watchSessionHeartbeat: WatchSessionBeacon;
};


export type MutationAddNodeToCollectionArgs = {
  collectionId: Scalars['String']['input'];
  nodeId: Scalars['String']['input'];
};


export type MutationCreateCollectionArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  filter?: InputMaybe<NodeFilter>;
  homePosition?: InputMaybe<Scalars['Int']['input']>;
  name: Scalars['String']['input'];
  pinned?: InputMaybe<Scalars['Boolean']['input']>;
  pinnedPosition?: InputMaybe<Scalars['Int']['input']>;
  resolverKind: CollectionResolverKind;
  showOnHome?: InputMaybe<Scalars['Boolean']['input']>;
  visibility: CollectionVisibility;
};


export type MutationCreateLibraryArgs = {
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
  pinned?: InputMaybe<Scalars['Boolean']['input']>;
};


export type MutationCreateUserInviteArgs = {
  libraryIds: Array<Scalars['String']['input']>;
  permissions: Scalars['Int']['input'];
  username: Scalars['String']['input'];
};


export type MutationDeleteCollectionArgs = {
  collectionId: Scalars['String']['input'];
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


export type MutationLeaveWatchSessionArgs = {
  playerId: Scalars['String']['input'];
  sessionId: Scalars['String']['input'];
};


export type MutationResetUserInviteArgs = {
  userId: Scalars['String']['input'];
};


export type MutationSetPreferredAudioArgs = {
  disposition?: InputMaybe<TrackDispositionPreference>;
  language?: InputMaybe<Scalars['String']['input']>;
};


export type MutationSetPreferredSubtitleArgs = {
  disposition?: InputMaybe<TrackDispositionPreference>;
  language?: InputMaybe<Scalars['String']['input']>;
};


export type MutationSignupArgs = {
  inviteCode?: InputMaybe<Scalars['String']['input']>;
  password: Scalars['String']['input'];
  permissions?: InputMaybe<Scalars['Int']['input']>;
  username: Scalars['String']['input'];
};


export type MutationUpdateCollectionArgs = {
  collectionId: Scalars['String']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
  filter?: InputMaybe<NodeFilter>;
  homePosition: Scalars['Int']['input'];
  name: Scalars['String']['input'];
  pinned: Scalars['Boolean']['input'];
  pinnedPosition: Scalars['Int']['input'];
  resolverKind: CollectionResolverKind;
  showOnHome: Scalars['Boolean']['input'];
  visibility: CollectionVisibility;
};


export type MutationUpdateLibraryArgs = {
  libraryId: Scalars['String']['input'];
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
  pinned: Scalars['Boolean']['input'];
};


export type MutationUpdateUserArgs = {
  libraryIds: Array<Scalars['String']['input']>;
  permissions: Scalars['Int']['input'];
  userId: Scalars['String']['input'];
  username: Scalars['String']['input'];
};


export type MutationUpdateWatchProgressArgs = {
  fileId: Scalars['String']['input'];
  progressPercent: Scalars['Float']['input'];
  userId?: InputMaybe<Scalars['String']['input']>;
};


export type MutationWatchSessionActionArgs = {
  input: WatchSessionActionInput;
};


export type MutationWatchSessionHeartbeatArgs = {
  input: WatchSessionHeartbeatInput;
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
  unavailableAt: Maybe<Scalars['Int']['output']>;
  unplayedCount: Maybe<Scalars['Int']['output']>;
  updatedAt: Scalars['Int']['output'];
  watchProgress: Maybe<WatchProgress>;
};

export enum NodeAvailability {
  Available = 'AVAILABLE',
  Both = 'BOTH',
  Unavailable = 'UNAVAILABLE'
}

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
  availability?: InputMaybe<NodeAvailability>;
  continueWatching?: InputMaybe<Scalars['Boolean']['input']>;
  kinds?: InputMaybe<Array<NodeKind>>;
  libraryId?: InputMaybe<Scalars['String']['input']>;
  orderBy?: InputMaybe<OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  parentId?: InputMaybe<Scalars['String']['input']>;
  rootId?: InputMaybe<Scalars['String']['input']>;
  searchTerm?: InputMaybe<Scalars['String']['input']>;
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
  displayDetail: Maybe<Scalars['String']['output']>;
  displayName: Scalars['String']['output'];
  durationSeconds: Maybe<Scalars['Int']['output']>;
  episodeNumber: Maybe<Scalars['Int']['output']>;
  fileSizeBytes: Maybe<Scalars['Int']['output']>;
  firstAired: Maybe<Scalars['Int']['output']>;
  fps: Maybe<Scalars['Float']['output']>;
  hasSubtitles: Maybe<Scalars['Boolean']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  lastAired: Maybe<Scalars['Int']['output']>;
  posterImage: Maybe<Asset>;
  rating: Maybe<Scalars['Float']['output']>;
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
  FirstAired = 'FIRST_AIRED',
  LastAired = 'LAST_AIRED',
  Order = 'ORDER',
  Rating = 'RATING',
  WatchProgressUpdatedAt = 'WATCH_PROGRESS_UPDATED_AT'
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
  collection: Maybe<Collection>;
  collections: Array<Collection>;
  home: HomeView;
  libraries: Array<Library>;
  library: Library;
  listFiles: Array<Scalars['String']['output']>;
  node: Node;
  nodeList: NodeConnection;
  users: Array<User>;
  viewer: Maybe<User>;
  watchSession: Maybe<WatchSession>;
  watchSessions: Array<WatchSession>;
};


export type QueryCollectionArgs = {
  collectionId: Scalars['String']['input'];
};


export type QueryCollectionsArgs = {
  pinned?: InputMaybe<Scalars['Boolean']['input']>;
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


export type QueryWatchSessionArgs = {
  sessionId: Scalars['String']['input'];
};

export type RecommendedTrack = {
  __typename: 'RecommendedTrack';
  enabled: Scalars['Boolean']['output'];
  manifestIndex: Scalars['Int']['output'];
  trackType: TrackType;
};

export type SubscriptionRoot = {
  __typename: 'SubscriptionRoot';
  contentUpdates: ContentUpdateEvent;
  watchSessionBeacons: WatchSessionBeacon;
};


export type SubscriptionRootWatchSessionBeaconsArgs = {
  playerId: Scalars['String']['input'];
  sessionId: Scalars['String']['input'];
};

export type TimelinePreviewSheet = {
  __typename: 'TimelinePreviewSheet';
  asset: Asset;
  endMs: Scalars['Int']['output'];
  positionMs: Scalars['Int']['output'];
  sheetGapSize: Scalars['Int']['output'];
  sheetIntervalMs: Scalars['Int']['output'];
};

export enum TrackDispositionPreference {
  Commentary = 'COMMENTARY',
  Normal = 'NORMAL',
  Sdh = 'SDH'
}

export type TrackInfo = {
  __typename: 'TrackInfo';
  displayName: Scalars['String']['output'];
  /** null if forced or unparseable */
  disposition: Maybe<TrackDispositionPreference>;
  isForced: Scalars['Boolean']['output'];
  /** iso 639 language code, null if unparseable */
  language: Maybe<Scalars['String']['output']>;
  /** 0-based index within type (maps to HLS.js index directly) */
  manifestIndex: Scalars['Int']['output'];
  /** original ffprobe stream index */
  trackIndex: Scalars['Int']['output'];
  trackType: TrackType;
};

export enum TrackType {
  Audio = 'AUDIO',
  Subtitle = 'SUBTITLE'
}

export type User = {
  __typename: 'User';
  createdAt: Scalars['Int']['output'];
  id: Scalars['String']['output'];
  inviteCode: Maybe<Scalars['String']['output']>;
  lastSeenAt: Maybe<Scalars['Int']['output']>;
  libraries: Array<Library>;
  permissions: Scalars['Int']['output'];
  preferredAudioDisposition: Maybe<Scalars['String']['output']>;
  preferredAudioLanguage: Maybe<Scalars['String']['output']>;
  preferredSubtitleDisposition: Maybe<Scalars['String']['output']>;
  preferredSubtitleLanguage: Maybe<Scalars['String']['output']>;
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

export type WatchSession = {
  __typename: 'WatchSession';
  basePositionMs: Scalars['Int']['output'];
  baseTimeMs: Scalars['Float']['output'];
  createdAt: Scalars['Int']['output'];
  currentPositionMs: Scalars['Int']['output'];
  effectiveState: EffectiveWatchSessionState;
  file: Maybe<File>;
  fileId: Scalars['String']['output'];
  id: Scalars['String']['output'];
  intent: WatchSessionIntent;
  mode: WatchSessionMode;
  node: Maybe<Node>;
  nodeId: Scalars['String']['output'];
  players: Array<WatchSessionPlayer>;
  revision: Scalars['Int']['output'];
  updatedAt: Scalars['Int']['output'];
};

export type WatchSessionActionInput = {
  kind: WatchSessionActionKind;
  nodeId?: InputMaybe<Scalars['String']['input']>;
  playerId: Scalars['String']['input'];
  positionMs?: InputMaybe<Scalars['Int']['input']>;
  sessionId: Scalars['String']['input'];
  targetPlayerId?: InputMaybe<Scalars['String']['input']>;
};

export enum WatchSessionActionKind {
  Pause = 'PAUSE',
  Play = 'PLAY',
  RemovePlayer = 'REMOVE_PLAYER',
  Seek = 'SEEK',
  SwitchItem = 'SWITCH_ITEM'
}

export type WatchSessionBeacon = {
  __typename: 'WatchSessionBeacon';
  basePositionMs: Scalars['Int']['output'];
  baseTimeMs: Scalars['Float']['output'];
  effectiveState: EffectiveWatchSessionState;
  fileId: Scalars['String']['output'];
  intent: WatchSessionIntent;
  mode: WatchSessionMode;
  nodeId: Scalars['String']['output'];
  players: Array<WatchSessionPlayer>;
  revision: Scalars['Int']['output'];
  sessionId: Scalars['String']['output'];
};

export type WatchSessionHeartbeatInput = {
  basePositionMs: Scalars['Int']['input'];
  baseTimeMs: Scalars['Float']['input'];
  isBuffering: Scalars['Boolean']['input'];
  playerId: Scalars['String']['input'];
  recovery: WatchSessionRecoveryInput;
  sessionId: Scalars['String']['input'];
};

export enum WatchSessionIntent {
  Paused = 'PAUSED',
  Playing = 'PLAYING'
}

export enum WatchSessionMode {
  Advisory = 'ADVISORY',
  Synced = 'SYNCED'
}

export type WatchSessionPlayer = {
  __typename: 'WatchSessionPlayer';
  basePositionMs: Scalars['Int']['output'];
  baseTimeMs: Scalars['Float']['output'];
  canRemove: Scalars['Boolean']['output'];
  id: Scalars['String']['output'];
  isBuffering: Scalars['Boolean']['output'];
  isInactive: Scalars['Boolean']['output'];
  joinedAt: Scalars['Int']['output'];
  lastReportMs: Scalars['Float']['output'];
  sessionId: Scalars['String']['output'];
  updatedAt: Scalars['Int']['output'];
  user: Maybe<User>;
  userId: Scalars['String']['output'];
};

export type WatchSessionRecoveryInput = {
  basePositionMs: Scalars['Int']['input'];
  baseTimeMs: Scalars['Float']['input'];
  fileId: Scalars['String']['input'];
  intent: WatchSessionIntent;
  nodeId: Scalars['String']['input'];
};

export type GetActivitiesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetActivitiesQuery = { activities: Array<{ __typename: 'Activity', taskType: string, title: string, current: number | null, total: number | null, progressPercent: number | null }> };

export type EditableCollectionsQueryVariables = Exact<{ [key: string]: never; }>;


export type EditableCollectionsQuery = { collections: Array<{ __typename: 'Collection', id: string, name: string, canEdit: boolean, resolverKind: CollectionResolverKind }> };

export type CreatePrivateCollectionMutationVariables = Exact<{
  name: Scalars['String']['input'];
  resolverKind: CollectionResolverKind;
  visibility: CollectionVisibility;
}>;


export type CreatePrivateCollectionMutation = { createCollection: { __typename: 'Collection', id: string, name: string } };

export type AddNodeToCollectionMutationVariables = Exact<{
  collectionId: Scalars['String']['input'];
  nodeId: Scalars['String']['input'];
}>;


export type AddNodeToCollectionMutation = { addNodeToCollection: { __typename: 'Collection', id: string, name: string } };

export type CollectionNodeCardFragment = (
  { __typename: 'Node', id: string, kind: NodeKind, libraryId: string, unavailableAt: number | null, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, firstAired: number | null, lastAired: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null, nextPlayable: { __typename: 'Node', id: string, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'CollectionNodeCardFragment' };

export type CollectionShelfFragment = { __typename: 'Collection', id: string, name: string, nodeList: { __typename: 'NodeConnection', nodes: Array<(
      { __typename: 'Node', id: string }
      & { ' $fragmentRefs'?: { 'NodePosterFragment': NodePosterFragment } }
    )> } } & { ' $fragmentName'?: 'CollectionShelfFragment' };

export type ContentUpdatesSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type ContentUpdatesSubscription = { contentUpdates: ContentUpdateEvent };

export type GetFilesQueryVariables = Exact<{
  path: Scalars['String']['input'];
}>;


export type GetFilesQuery = { listFiles: Array<string> };

export type EpisodeCardFragment = (
  { __typename: 'Node', id: string, unavailableAt: number | null, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, firstAired: number | null, runtimeMinutes: number | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'EpisodeCardFragment' };

export type ImageAssetFragment = { __typename: 'Asset', id: string, signedUrl: string, thumbhash: string | null } & { ' $fragmentName'?: 'ImageAssetFragment' };

export type RunImportWatchStatesMutationVariables = Exact<{
  input: ImportWatchStatesInput;
}>;


export type RunImportWatchStatesMutation = { importWatchStates: { __typename: 'ImportWatchStatesResult', dryRun: boolean, totalRows: number, matchedRows: number, unmatchedRows: number, conflictRows: number, willInsert: number, willOverwrite: number, imported: number, skipped: number, conflicts: Array<{ __typename: 'ImportWatchStateConflict', rowIndex: number, sourceItemId: string | null, title: string | null, itemId: string, existingProgressPercent: number, importedProgressPercent: number, reason: string }>, unmatched: Array<{ __typename: 'ImportWatchStateUnmatched', rowIndex: number, sourceItemId: string | null, title: string | null, reason: string, ambiguous: boolean }> } };

export type NodePageQueryVariables = Exact<{
  after?: InputMaybe<Scalars['String']['input']>;
  first: Scalars['Int']['input'];
  filter: NodeFilter;
}>;


export type NodePageQuery = { nodeList: { __typename: 'NodeConnection', edges: Array<{ __typename: 'NodeEdge', node: (
        { __typename: 'Node', id: string }
        & { ' $fragmentRefs'?: { 'NodePosterFragment': NodePosterFragment;'EpisodeCardFragment': EpisodeCardFragment } }
      ) }>, pageInfo: { __typename: 'PageInfo', endCursor: string | null, hasNextPage: boolean } } };

export type NodePosterFragment = (
  { __typename: 'Node', id: string, kind: NodeKind, libraryId: string, unavailableAt: number | null, unplayedCount: number | null, seasonCount: number, episodeCount: number, seasonNumber: number | null, episodeNumber: number | null, properties: { __typename: 'NodeProperties', displayName: string, displayDetail: string | null, firstAired: number | null, lastAired: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, nextPlayable: { __typename: 'Node', id: string, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'NodePosterFragment' };

export type PlayerTimelinePreviewSheetFragment = { __typename: 'TimelinePreviewSheet', positionMs: number, endMs: number, sheetIntervalMs: number, sheetGapSize: number, asset: { __typename: 'Asset', id: string, signedUrl: string, width: number | null, height: number | null } } & { ' $fragmentName'?: 'PlayerTimelinePreviewSheetFragment' };

export type ItemPlaybackQueryVariables = Exact<{
  itemId: Scalars['String']['input'];
}>;


export type ItemPlaybackQuery = { node: (
    { __typename: 'Node', id: string, libraryId: string, kind: NodeKind, properties: { __typename: 'NodeProperties', displayName: string, seasonNumber: number | null, episodeNumber: number | null, runtimeMinutes: number | null, firstAired: number | null, lastAired: number | null }, root: { __typename: 'Node', libraryId: string, properties: { __typename: 'NodeProperties', displayName: string } } | null, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null, file: { __typename: 'File', id: string, tracks: Array<{ __typename: 'TrackInfo', trackIndex: number, manifestIndex: number, trackType: TrackType, displayName: string, language: string | null, disposition: TrackDispositionPreference | null, isForced: boolean }>, recommendedTracks: Array<{ __typename: 'RecommendedTrack', manifestIndex: number, trackType: TrackType, enabled: boolean }>, segments: Array<{ __typename: 'FileSegment', kind: FileSegmentKind, startMs: number, endMs: number }>, timelinePreview: Array<(
        { __typename: 'TimelinePreviewSheet' }
        & { ' $fragmentRefs'?: { 'PlayerTimelinePreviewSheetFragment': PlayerTimelinePreviewSheetFragment } }
      )> } | null, previousPlayable: { __typename: 'Node', id: string, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, thumbnailImage: (
          { __typename: 'Asset' }
          & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
        ) | null } } | null, nextPlayable: { __typename: 'Node', id: string, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, thumbnailImage: (
          { __typename: 'Asset' }
          & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
        ) | null } } | null }
    & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
  ) };

export type UpdateWatchStateMutationVariables = Exact<{
  fileId: Scalars['String']['input'];
  progressPercent: Scalars['Float']['input'];
}>;


export type UpdateWatchStateMutation = { updateWatchProgress: Array<{ __typename: 'WatchProgress', progressPercent: number, updatedAt: number }> };

export type SetPreferredAudioMutationVariables = Exact<{
  language?: InputMaybe<Scalars['String']['input']>;
  disposition?: InputMaybe<TrackDispositionPreference>;
}>;


export type SetPreferredAudioMutation = { setPreferredAudio: { __typename: 'User', id: string, preferredAudioLanguage: string | null, preferredAudioDisposition: string | null } };

export type SetPreferredSubtitleMutationVariables = Exact<{
  language?: InputMaybe<Scalars['String']['input']>;
  disposition?: InputMaybe<TrackDispositionPreference>;
}>;


export type SetPreferredSubtitleMutation = { setPreferredSubtitle: { __typename: 'User', id: string, preferredSubtitleLanguage: string | null, preferredSubtitleDisposition: string | null } };

export type WatchSessionSummaryFragment = { __typename: 'WatchSession', id: string, nodeId: string, fileId: string, mode: WatchSessionMode, intent: WatchSessionIntent, effectiveState: EffectiveWatchSessionState, currentPositionMs: number, basePositionMs: number, baseTimeMs: number, revision: number, players: Array<{ __typename: 'WatchSessionPlayer', id: string, userId: string, isBuffering: boolean, isInactive: boolean, canRemove: boolean, user: { __typename: 'User', id: string, username: string } | null }> } & { ' $fragmentName'?: 'WatchSessionSummaryFragment' };

export type WatchSessionBeaconFragmentFragment = { __typename: 'WatchSessionBeacon', sessionId: string, nodeId: string, fileId: string, mode: WatchSessionMode, intent: WatchSessionIntent, effectiveState: EffectiveWatchSessionState, basePositionMs: number, baseTimeMs: number, revision: number, players: Array<{ __typename: 'WatchSessionPlayer', id: string, userId: string, isBuffering: boolean, isInactive: boolean, canRemove: boolean, user: { __typename: 'User', id: string, username: string } | null }> } & { ' $fragmentName'?: 'WatchSessionBeaconFragmentFragment' };

export type WatchSessionViewerQueryVariables = Exact<{ [key: string]: never; }>;


export type WatchSessionViewerQuery = { viewer: { __typename: 'User', id: string, permissions: number } | null };

export type GetWatchSessionQueryVariables = Exact<{
  sessionId: Scalars['String']['input'];
}>;


export type GetWatchSessionQuery = { watchSession: (
    { __typename: 'WatchSession' }
    & { ' $fragmentRefs'?: { 'WatchSessionSummaryFragment': WatchSessionSummaryFragment } }
  ) | null };

export type LeaveWatchSessionMutationVariables = Exact<{
  sessionId: Scalars['String']['input'];
  playerId: Scalars['String']['input'];
}>;


export type LeaveWatchSessionMutation = { leaveWatchSession: boolean };

export type WatchSessionHeartbeatMutationVariables = Exact<{
  input: WatchSessionHeartbeatInput;
}>;


export type WatchSessionHeartbeatMutation = { watchSessionHeartbeat: (
    { __typename: 'WatchSessionBeacon' }
    & { ' $fragmentRefs'?: { 'WatchSessionBeaconFragmentFragment': WatchSessionBeaconFragmentFragment } }
  ) };

export type WatchSessionActionMutationVariables = Exact<{
  input: WatchSessionActionInput;
}>;


export type WatchSessionActionMutation = { watchSessionAction: (
    { __typename: 'WatchSessionBeacon' }
    & { ' $fragmentRefs'?: { 'WatchSessionBeaconFragmentFragment': WatchSessionBeaconFragmentFragment } }
  ) };

export type WatchSessionBeaconsSubscriptionVariables = Exact<{
  sessionId: Scalars['String']['input'];
  playerId: Scalars['String']['input'];
}>;


export type WatchSessionBeaconsSubscription = { watchSessionBeacons: (
    { __typename: 'WatchSessionBeacon' }
    & { ' $fragmentRefs'?: { 'WatchSessionBeaconFragmentFragment': WatchSessionBeaconFragmentFragment } }
  ) };

export type SearchNodeResultFragment = (
  { __typename: 'Node', id: string, kind: NodeKind, libraryId: string, seasonCount: number, episodeCount: number, root: { __typename: 'Node', properties: { __typename: 'NodeProperties', displayName: string } } | null, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, firstAired: number | null, lastAired: number | null, runtimeMinutes: number | null, posterImage: (
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
  limit: Scalars['Int']['input'];
  kinds: Array<NodeKind> | NodeKind;
}>;


export type SearchMediaQuery = { nodeList: { __typename: 'NodeConnection', nodes: Array<(
      { __typename: 'Node' }
      & { ' $fragmentRefs'?: { 'SearchNodeResultFragment': SearchNodeResultFragment } }
    )> } };

export type SeasonCardFragment = (
  { __typename: 'Node', id: string, unavailableAt: number | null, unplayedCount: number | null, episodeCount: number, properties: { __typename: 'NodeProperties', displayName: string, seasonNumber: number | null, firstAired: number | null, lastAired: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, nextPlayable: { __typename: 'Node', id: string, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'SeasonCardFragment' };

export type LibraryCardFragment = { __typename: 'Library', id: string, name: string, path: string, pinned: boolean, createdAt: number, lastScannedAt: number | null } & { ' $fragmentName'?: 'LibraryCardFragment' };

export type GetLibrariesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetLibrariesQuery = { libraries: Array<(
    { __typename: 'Library', id: string }
    & { ' $fragmentRefs'?: { 'LibraryCardFragment': LibraryCardFragment } }
  )> };

export type CreateLibraryMutationVariables = Exact<{
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
  pinned: Scalars['Boolean']['input'];
}>;


export type CreateLibraryMutation = { createLibrary: (
    { __typename: 'Library' }
    & { ' $fragmentRefs'?: { 'LibraryCardFragment': LibraryCardFragment } }
  ) };

export type UpdateLibraryMutationVariables = Exact<{
  libraryId: Scalars['String']['input'];
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
  pinned: Scalars['Boolean']['input'];
}>;


export type UpdateLibraryMutation = { updateLibrary: (
    { __typename: 'Library' }
    & { ' $fragmentRefs'?: { 'LibraryCardFragment': LibraryCardFragment } }
  ) };

export type DeleteLibraryMutationVariables = Exact<{
  libraryId: Scalars['String']['input'];
}>;


export type DeleteLibraryMutation = { deleteLibrary: boolean };

export type SessionCardFragment = { __typename: 'WatchSession', id: string, updatedAt: number, currentPositionMs: number, effectiveState: EffectiveWatchSessionState, players: Array<{ __typename: 'WatchSessionPlayer', id: string, userId: string, user: { __typename: 'User', id: string, username: string, createdAt: number } | null }>, node: { __typename: 'Node', id: string, libraryId: string, properties: { __typename: 'NodeProperties', displayName: string, seasonNumber: number | null, episodeNumber: number | null, runtimeMinutes: number | null, firstAired: number | null, lastAired: number | null, posterImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, thumbnailImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null }, root: { __typename: 'Node', id: string, properties: { __typename: 'NodeProperties', displayName: string, posterImage: (
          { __typename: 'Asset' }
          & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
        ) | null } } | null } | null, file: { __typename: 'File', id: string, timelinePreview: Array<(
      { __typename: 'TimelinePreviewSheet' }
      & { ' $fragmentRefs'?: { 'PlayerTimelinePreviewSheetFragment': PlayerTimelinePreviewSheetFragment } }
    )> } | null } & { ' $fragmentName'?: 'SessionCardFragment' };

export type SettingsSessionsQueryVariables = Exact<{ [key: string]: never; }>;


export type SettingsSessionsQuery = { watchSessions: Array<(
    { __typename: 'WatchSession', id: string }
    & { ' $fragmentRefs'?: { 'SessionCardFragment': SessionCardFragment } }
  )> };

export type UsersManagementQueryVariables = Exact<{ [key: string]: never; }>;


export type UsersManagementQuery = { viewer: { __typename: 'User', id: string } | null, libraries: Array<{ __typename: 'Library', id: string, name: string, createdAt: number }>, users: Array<(
    { __typename: 'User', id: string }
    & { ' $fragmentRefs'?: { 'UserCardFragment': UserCardFragment } }
  )> };

export type CreateUserInviteMutationVariables = Exact<{
  username: Scalars['String']['input'];
  permissions: Scalars['Int']['input'];
  libraryIds: Array<Scalars['String']['input']> | Scalars['String']['input'];
}>;


export type CreateUserInviteMutation = { createUserInvite: (
    { __typename: 'User' }
    & { ' $fragmentRefs'?: { 'UserCardFragment': UserCardFragment } }
  ) };

export type UpdateUserMutationVariables = Exact<{
  userId: Scalars['String']['input'];
  username: Scalars['String']['input'];
  permissions: Scalars['Int']['input'];
  libraryIds: Array<Scalars['String']['input']> | Scalars['String']['input'];
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

export type UserCardFragment = { __typename: 'User', id: string, username: string, inviteCode: string | null, permissions: number, createdAt: number, lastSeenAt: number | null, libraries: Array<{ __typename: 'Library', id: string }> } & { ' $fragmentName'?: 'UserCardFragment' };

export type SidebarNavigationQueryVariables = Exact<{ [key: string]: never; }>;


export type SidebarNavigationQuery = { libraries: Array<{ __typename: 'Library', id: string, name: string, createdAt: number, pinned: boolean }>, collections: Array<{ __typename: 'Collection', id: string, name: string }> };

export type SidebarViewerQueryVariables = Exact<{ [key: string]: never; }>;


export type SidebarViewerQuery = { viewer: { __typename: 'User', id: string, permissions: number } | null };

export type GetPathForNodeFragment = { __typename: 'Node', id: string, libraryId: string } & { ' $fragmentName'?: 'GetPathForNodeFragment' };

export type CollectionPageQueryVariables = Exact<{
  collectionId: Scalars['String']['input'];
  after?: InputMaybe<Scalars['String']['input']>;
  first: Scalars['Int']['input'];
}>;


export type CollectionPageQuery = { collection: { __typename: 'Collection', id: string, name: string, description: string | null, itemCount: number, canDelete: boolean, nodeList: { __typename: 'NodeConnection', nodes: Array<(
        { __typename: 'Node', id: string }
        & { ' $fragmentRefs'?: { 'NodePosterFragment': NodePosterFragment } }
      )>, pageInfo: { __typename: 'PageInfo', endCursor: string | null, hasNextPage: boolean } } } | null };

export type DeleteCollectionMutationVariables = Exact<{
  collectionId: Scalars['String']['input'];
}>;


export type DeleteCollectionMutation = { deleteCollection: boolean };

export type CollectionsIndexQueryVariables = Exact<{ [key: string]: never; }>;


export type CollectionsIndexQuery = { collections: Array<{ __typename: 'Collection', id: string, name: string, description: string | null, itemCount: number, visibility: CollectionVisibility, createdBy: { __typename: 'User', username: string } | null }> };

export type HomeCollectionsQueryVariables = Exact<{ [key: string]: never; }>;


export type HomeCollectionsQuery = { home: { __typename: 'HomeView', sections: Array<(
      { __typename: 'Collection', id: string }
      & { ' $fragmentRefs'?: { 'CollectionShelfFragment': CollectionShelfFragment } }
    )> } };

export type GetNodeByIdQueryVariables = Exact<{
  nodeId: Scalars['String']['input'];
}>;


export type GetNodeByIdQuery = { node: (
    { __typename: 'Node', id: string, libraryId: string, kind: NodeKind, unavailableAt: number | null, seasonNumber: number | null, episodeNumber: number | null, unplayedCount: number | null, episodeCount: number, parent: (
      { __typename: 'Node', id: string, libraryId: string, properties: { __typename: 'NodeProperties', displayName: string } }
      & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
    ) | null, root: { __typename: 'Node', id: string, properties: { __typename: 'NodeProperties', displayName: string } } | null, children: Array<(
      { __typename: 'Node', id: string, kind: NodeKind, order: number, properties: { __typename: 'NodeProperties', seasonNumber: number | null } }
      & { ' $fragmentRefs'?: { 'SeasonCardFragment': SeasonCardFragment } }
    )>, properties: { __typename: 'NodeProperties', displayName: string, firstAired: number | null, lastAired: number | null, runtimeMinutes: number | null, description: string | null, posterImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, backgroundImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, thumbnailImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null }, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null, nextPlayable: { __typename: 'Node', id: string, watchProgress: { __typename: 'WatchProgress', id: string, progressPercent: number, completed: boolean, updatedAt: number } | null } | null, previousPlayable: { __typename: 'Node', id: string } | null }
    & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
  ) };

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
export const CollectionNodeCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CollectionNodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<CollectionNodeCardFragment, unknown>;
export const NodePosterFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<NodePosterFragment, unknown>;
export const CollectionShelfFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CollectionShelf"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Collection"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"12"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<CollectionShelfFragment, unknown>;
export const EpisodeCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<EpisodeCardFragment, unknown>;
export const WatchSessionSummaryFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"WatchSessionSummary"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSession"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"fileId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"intent"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"currentPositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"basePositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"baseTimeMs"}},{"kind":"Field","name":{"kind":"Name","value":"revision"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}},{"kind":"Field","name":{"kind":"Name","value":"isBuffering"}},{"kind":"Field","name":{"kind":"Name","value":"isInactive"}},{"kind":"Field","name":{"kind":"Name","value":"canRemove"}}]}}]}}]} as unknown as DocumentNode<WatchSessionSummaryFragment, unknown>;
export const WatchSessionBeaconFragmentFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"WatchSessionBeaconFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSessionBeacon"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sessionId"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"fileId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"intent"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"basePositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"baseTimeMs"}},{"kind":"Field","name":{"kind":"Name","value":"revision"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}},{"kind":"Field","name":{"kind":"Name","value":"isBuffering"}},{"kind":"Field","name":{"kind":"Name","value":"isInactive"}},{"kind":"Field","name":{"kind":"Name","value":"canRemove"}}]}}]}}]} as unknown as DocumentNode<WatchSessionBeaconFragmentFragment, unknown>;
export const SearchNodeResultFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchNodeResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<SearchNodeResultFragment, unknown>;
export const SeasonCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SeasonCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<SeasonCardFragment, unknown>;
export const LibraryCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<LibraryCardFragment, unknown>;
export const PlayerTimelinePreviewSheetFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}}]} as unknown as DocumentNode<PlayerTimelinePreviewSheetFragment, unknown>;
export const SessionCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SessionCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSession"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"currentPositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"file"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"timelinePreview"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}}]} as unknown as DocumentNode<SessionCardFragment, unknown>;
export const UserCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UserCardFragment, unknown>;
export const GetActivitiesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetActivities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskType"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"current"}},{"kind":"Field","name":{"kind":"Name","value":"total"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}}]}}]}}]} as unknown as DocumentNode<GetActivitiesQuery, GetActivitiesQueryVariables>;
export const EditableCollectionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"EditableCollections"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"collections"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"canEdit"}},{"kind":"Field","name":{"kind":"Name","value":"resolverKind"}}]}}]}}]} as unknown as DocumentNode<EditableCollectionsQuery, EditableCollectionsQueryVariables>;
export const CreatePrivateCollectionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreatePrivateCollection"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"resolverKind"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CollectionResolverKind"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"visibility"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"CollectionVisibility"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createCollection"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"resolverKind"},"value":{"kind":"Variable","name":{"kind":"Name","value":"resolverKind"}}},{"kind":"Argument","name":{"kind":"Name","value":"visibility"},"value":{"kind":"Variable","name":{"kind":"Name","value":"visibility"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<CreatePrivateCollectionMutation, CreatePrivateCollectionMutationVariables>;
export const AddNodeToCollectionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"AddNodeToCollection"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"collectionId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"addNodeToCollection"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"collectionId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"collectionId"}}},{"kind":"Argument","name":{"kind":"Name","value":"nodeId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<AddNodeToCollectionMutation, AddNodeToCollectionMutationVariables>;
export const ContentUpdatesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"ContentUpdates"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"contentUpdates"}}]}}]} as unknown as DocumentNode<ContentUpdatesSubscription, ContentUpdatesSubscriptionVariables>;
export const GetFilesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetFiles"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"listFiles"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}]}]}}]} as unknown as DocumentNode<GetFilesQuery, GetFilesQueryVariables>;
export const RunImportWatchStatesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"RunImportWatchStates"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ImportWatchStatesInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"importWatchStates"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"totalRows"}},{"kind":"Field","name":{"kind":"Name","value":"matchedRows"}},{"kind":"Field","name":{"kind":"Name","value":"unmatchedRows"}},{"kind":"Field","name":{"kind":"Name","value":"conflictRows"}},{"kind":"Field","name":{"kind":"Name","value":"willInsert"}},{"kind":"Field","name":{"kind":"Name","value":"willOverwrite"}},{"kind":"Field","name":{"kind":"Name","value":"imported"}},{"kind":"Field","name":{"kind":"Name","value":"skipped"}},{"kind":"Field","name":{"kind":"Name","value":"conflicts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rowIndex"}},{"kind":"Field","name":{"kind":"Name","value":"sourceItemId"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"itemId"}},{"kind":"Field","name":{"kind":"Name","value":"existingProgressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"importedProgressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"reason"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unmatched"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rowIndex"}},{"kind":"Field","name":{"kind":"Name","value":"sourceItemId"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"reason"}},{"kind":"Field","name":{"kind":"Name","value":"ambiguous"}}]}}]}}]}}]} as unknown as DocumentNode<RunImportWatchStatesMutation, RunImportWatchStatesMutationVariables>;
export const NodePageDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"NodePage"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"first"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"NodeFilter"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}},{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"Variable","name":{"kind":"Name","value":"first"}}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"EpisodeCard"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<NodePageQuery, NodePageQueryVariables>;
export const ItemPlaybackDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ItemPlayback"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"nodeId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"file"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"tracks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"trackIndex"}},{"kind":"Field","name":{"kind":"Name","value":"manifestIndex"}},{"kind":"Field","name":{"kind":"Name","value":"trackType"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"language"}},{"kind":"Field","name":{"kind":"Name","value":"disposition"}},{"kind":"Field","name":{"kind":"Name","value":"isForced"}}]}},{"kind":"Field","name":{"kind":"Name","value":"recommendedTracks"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"manifestIndex"}},{"kind":"Field","name":{"kind":"Name","value":"trackType"}},{"kind":"Field","name":{"kind":"Name","value":"enabled"}}]}},{"kind":"Field","name":{"kind":"Name","value":"segments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"startMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}}]}},{"kind":"Field","name":{"kind":"Name","value":"timelinePreview"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"previousPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}}]}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<ItemPlaybackQuery, ItemPlaybackQueryVariables>;
export const UpdateWatchStateDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateWatchState"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"fileId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"progressPercent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Float"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateWatchProgress"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"fileId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"fileId"}}},{"kind":"Argument","name":{"kind":"Name","value":"progressPercent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"progressPercent"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}}]} as unknown as DocumentNode<UpdateWatchStateMutation, UpdateWatchStateMutationVariables>;
export const SetPreferredAudioDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"SetPreferredAudio"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"language"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"disposition"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"TrackDispositionPreference"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"setPreferredAudio"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"language"},"value":{"kind":"Variable","name":{"kind":"Name","value":"language"}}},{"kind":"Argument","name":{"kind":"Name","value":"disposition"},"value":{"kind":"Variable","name":{"kind":"Name","value":"disposition"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"preferredAudioLanguage"}},{"kind":"Field","name":{"kind":"Name","value":"preferredAudioDisposition"}}]}}]}}]} as unknown as DocumentNode<SetPreferredAudioMutation, SetPreferredAudioMutationVariables>;
export const SetPreferredSubtitleDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"SetPreferredSubtitle"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"language"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"disposition"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"TrackDispositionPreference"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"setPreferredSubtitle"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"language"},"value":{"kind":"Variable","name":{"kind":"Name","value":"language"}}},{"kind":"Argument","name":{"kind":"Name","value":"disposition"},"value":{"kind":"Variable","name":{"kind":"Name","value":"disposition"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"preferredSubtitleLanguage"}},{"kind":"Field","name":{"kind":"Name","value":"preferredSubtitleDisposition"}}]}}]}}]} as unknown as DocumentNode<SetPreferredSubtitleMutation, SetPreferredSubtitleMutationVariables>;
export const WatchSessionViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"WatchSessionViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<WatchSessionViewerQuery, WatchSessionViewerQueryVariables>;
export const GetWatchSessionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetWatchSession"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"sessionId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"watchSession"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"sessionId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"sessionId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"WatchSessionSummary"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"WatchSessionSummary"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSession"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"fileId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"intent"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"currentPositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"basePositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"baseTimeMs"}},{"kind":"Field","name":{"kind":"Name","value":"revision"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}},{"kind":"Field","name":{"kind":"Name","value":"isBuffering"}},{"kind":"Field","name":{"kind":"Name","value":"isInactive"}},{"kind":"Field","name":{"kind":"Name","value":"canRemove"}}]}}]}}]} as unknown as DocumentNode<GetWatchSessionQuery, GetWatchSessionQueryVariables>;
export const LeaveWatchSessionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"LeaveWatchSession"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"sessionId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"playerId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"leaveWatchSession"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"sessionId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"sessionId"}}},{"kind":"Argument","name":{"kind":"Name","value":"playerId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"playerId"}}}]}]}}]} as unknown as DocumentNode<LeaveWatchSessionMutation, LeaveWatchSessionMutationVariables>;
export const WatchSessionHeartbeatDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"WatchSessionHeartbeat"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSessionHeartbeatInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"watchSessionHeartbeat"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"WatchSessionBeaconFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"WatchSessionBeaconFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSessionBeacon"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sessionId"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"fileId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"intent"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"basePositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"baseTimeMs"}},{"kind":"Field","name":{"kind":"Name","value":"revision"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}},{"kind":"Field","name":{"kind":"Name","value":"isBuffering"}},{"kind":"Field","name":{"kind":"Name","value":"isInactive"}},{"kind":"Field","name":{"kind":"Name","value":"canRemove"}}]}}]}}]} as unknown as DocumentNode<WatchSessionHeartbeatMutation, WatchSessionHeartbeatMutationVariables>;
export const WatchSessionActionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"WatchSessionAction"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSessionActionInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"watchSessionAction"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"WatchSessionBeaconFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"WatchSessionBeaconFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSessionBeacon"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sessionId"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"fileId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"intent"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"basePositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"baseTimeMs"}},{"kind":"Field","name":{"kind":"Name","value":"revision"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}},{"kind":"Field","name":{"kind":"Name","value":"isBuffering"}},{"kind":"Field","name":{"kind":"Name","value":"isInactive"}},{"kind":"Field","name":{"kind":"Name","value":"canRemove"}}]}}]}}]} as unknown as DocumentNode<WatchSessionActionMutation, WatchSessionActionMutationVariables>;
export const WatchSessionBeaconsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"subscription","name":{"kind":"Name","value":"WatchSessionBeacons"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"sessionId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"playerId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"watchSessionBeacons"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"sessionId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"sessionId"}}},{"kind":"Argument","name":{"kind":"Name","value":"playerId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"playerId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"WatchSessionBeaconFragment"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"WatchSessionBeaconFragment"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSessionBeacon"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sessionId"}},{"kind":"Field","name":{"kind":"Name","value":"nodeId"}},{"kind":"Field","name":{"kind":"Name","value":"fileId"}},{"kind":"Field","name":{"kind":"Name","value":"mode"}},{"kind":"Field","name":{"kind":"Name","value":"intent"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"basePositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"baseTimeMs"}},{"kind":"Field","name":{"kind":"Name","value":"revision"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}},{"kind":"Field","name":{"kind":"Name","value":"isBuffering"}},{"kind":"Field","name":{"kind":"Name","value":"isInactive"}},{"kind":"Field","name":{"kind":"Name","value":"canRemove"}}]}}]}}]} as unknown as DocumentNode<WatchSessionBeaconsSubscription, WatchSessionBeaconsSubscriptionVariables>;
export const SearchMediaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SearchMedia"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"query"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"limit"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"kinds"}},"type":{"kind":"NonNullType","type":{"kind":"ListType","type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"NodeKind"}}}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"Variable","name":{"kind":"Name","value":"limit"}}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"searchTerm"},"value":{"kind":"Variable","name":{"kind":"Name","value":"query"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"kinds"},"value":{"kind":"Variable","name":{"kind":"Name","value":"kinds"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SearchNodeResult"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchNodeResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<SearchMediaQuery, SearchMediaQueryVariables>;
export const GetLibrariesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLibraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<GetLibrariesQuery, GetLibrariesQueryVariables>;
export const CreateLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}},{"kind":"Argument","name":{"kind":"Name","value":"pinned"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<CreateLibraryMutation, CreateLibraryMutationVariables>;
export const UpdateLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}},{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}},{"kind":"Argument","name":{"kind":"Name","value":"pinned"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<UpdateLibraryMutation, UpdateLibraryMutationVariables>;
export const DeleteLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}}]}]}}]} as unknown as DocumentNode<DeleteLibraryMutation, DeleteLibraryMutationVariables>;
export const SettingsSessionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SettingsSessions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"watchSessions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"SessionCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SessionCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"WatchSession"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"currentPositionMs"}},{"kind":"Field","name":{"kind":"Name","value":"effectiveState"}},{"kind":"Field","name":{"kind":"Name","value":"players"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"userId"}},{"kind":"Field","name":{"kind":"Name","value":"user"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"file"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"timelinePreview"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"}}]}}]}}]}}]} as unknown as DocumentNode<SettingsSessionsQuery, SettingsSessionsQueryVariables>;
export const UsersManagementDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"UsersManagement"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"users"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UsersManagementQuery, UsersManagementQueryVariables>;
export const CreateUserInviteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateUserInvite"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryIds"}},"type":{"kind":"NonNullType","type":{"kind":"ListType","type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createUserInvite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"permissions"},"value":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}}},{"kind":"Argument","name":{"kind":"Name","value":"libraryIds"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryIds"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<CreateUserInviteMutation, CreateUserInviteMutationVariables>;
export const UpdateUserDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateUser"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryIds"}},"type":{"kind":"NonNullType","type":{"kind":"ListType","type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateUser"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}},{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"permissions"},"value":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}}},{"kind":"Argument","name":{"kind":"Name","value":"libraryIds"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryIds"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UpdateUserMutation, UpdateUserMutationVariables>;
export const ResetUserInviteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ResetUserInvite"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"resetUserInvite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<ResetUserInviteMutation, ResetUserInviteMutationVariables>;
export const DeleteUserDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteUser"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteUser"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}}]}]}}]} as unknown as DocumentNode<DeleteUserMutation, DeleteUserMutationVariables>;
export const SidebarNavigationDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SidebarNavigation"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}}]}},{"kind":"Field","name":{"kind":"Name","value":"collections"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"pinned"},"value":{"kind":"BooleanValue","value":true}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}}]} as unknown as DocumentNode<SidebarNavigationQuery, SidebarNavigationQueryVariables>;
export const SidebarViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SidebarViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<SidebarViewerQuery, SidebarViewerQueryVariables>;
export const CollectionPageDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"CollectionPage"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"collectionId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"first"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"collection"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"collectionId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"collectionId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"itemCount"}},{"kind":"Field","name":{"kind":"Name","value":"canDelete"}},{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}},{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"Variable","name":{"kind":"Name","value":"first"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<CollectionPageQuery, CollectionPageQueryVariables>;
export const DeleteCollectionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteCollection"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"collectionId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteCollection"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"collectionId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"collectionId"}}}]}]}}]} as unknown as DocumentNode<DeleteCollectionMutation, DeleteCollectionMutationVariables>;
export const CollectionsIndexDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"CollectionsIndex"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"collections"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"itemCount"}},{"kind":"Field","name":{"kind":"Name","value":"visibility"}},{"kind":"Field","name":{"kind":"Name","value":"createdBy"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"username"}}]}}]}}]}}]} as unknown as DocumentNode<CollectionsIndexQuery, CollectionsIndexQueryVariables>;
export const HomeCollectionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"HomeCollections"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"home"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sections"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"CollectionShelf"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"CollectionShelf"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Collection"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"12"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}}]}}]}}]} as unknown as DocumentNode<HomeCollectionsQuery, HomeCollectionsQueryVariables>;
export const GetNodeByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetNodeById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"nodeId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"children"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"order"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"SeasonCard"}}]}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"backgroundImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"previousPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SeasonCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<GetNodeByIdQuery, GetNodeByIdQueryVariables>;
export const PlaygroundViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PlaygroundViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<PlaygroundViewerQuery, PlaygroundViewerQueryVariables>;
export const SettingsViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SettingsViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<SettingsViewerQuery, SettingsViewerQueryVariables>;
export const SignupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Signup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"password"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"inviteCode"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"signup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"password"},"value":{"kind":"Variable","name":{"kind":"Name","value":"password"}}},{"kind":"Argument","name":{"kind":"Name","value":"inviteCode"},"value":{"kind":"Variable","name":{"kind":"Name","value":"inviteCode"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}}]}}]} as unknown as DocumentNode<SignupMutation, SignupMutationVariables>;