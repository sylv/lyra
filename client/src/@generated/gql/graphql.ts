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
  id: Scalars['Int']['output'];
  mimeType: Maybe<Scalars['String']['output']>;
  sizeBytes: Maybe<Scalars['Int']['output']>;
  sourceUrl: Maybe<Scalars['String']['output']>;
  thumbhash: Maybe<Scalars['String']['output']>;
  width: Maybe<Scalars['Int']['output']>;
};

export type File = {
  __typename: 'File';
  discoveredAt: Scalars['Int']['output'];
  editionName: Maybe<Scalars['String']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  id: Scalars['Int']['output'];
  libraryId: Scalars['Int']['output'];
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

export enum ItemKind {
  Episode = 'EPISODE',
  Movie = 'MOVIE'
}

export type ItemNode = {
  __typename: 'ItemNode';
  createdAt: Scalars['Int']['output'];
  episodeNumber: Maybe<Scalars['Int']['output']>;
  file: Maybe<File>;
  id: Scalars['String']['output'];
  kind: ItemKind;
  lastAddedAt: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  nextItem: Maybe<ItemNode>;
  order: Scalars['Int']['output'];
  parent: Maybe<RootNode>;
  previousItem: Maybe<ItemNode>;
  properties: ItemNodeProperties;
  rootId: Scalars['String']['output'];
  seasonId: Maybe<Scalars['String']['output']>;
  updatedAt: Scalars['Int']['output'];
  watchProgress: Maybe<WatchProgress>;
};

export type ItemNodeConnection = {
  __typename: 'ItemNodeConnection';
  /** A list of edges. */
  edges: Array<ItemNodeEdge>;
  /** A list of nodes. */
  nodes: Array<ItemNode>;
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
};

/** An edge in a connection. */
export type ItemNodeEdge = {
  __typename: 'ItemNodeEdge';
  /** A cursor for use in pagination */
  cursor: Scalars['String']['output'];
  /** The item at the end of the edge */
  node: ItemNode;
};

export type ItemNodeFilter = {
  orderBy?: InputMaybe<OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  rootId: Scalars['String']['input'];
  seasonNumbers?: InputMaybe<Array<Scalars['Int']['input']>>;
  watched?: InputMaybe<Scalars['Boolean']['input']>;
};

export type ItemNodeProperties = {
  __typename: 'ItemNodeProperties';
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

export type Library = {
  __typename: 'Library';
  createdAt: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  lastScannedAt: Maybe<Scalars['Int']['output']>;
  name: Scalars['String']['output'];
  path: Scalars['String']['output'];
};

export type Mutation = {
  __typename: 'Mutation';
  createLibrary: Library;
  importWatchStates: ImportWatchStatesResult;
  signup: User;
  updateWatchProgress: Array<WatchProgress>;
};


export type MutationCreateLibraryArgs = {
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
};


export type MutationImportWatchStatesArgs = {
  input: ImportWatchStatesInput;
};


export type MutationSignupArgs = {
  inviteCode?: InputMaybe<Scalars['String']['input']>;
  password: Scalars['String']['input'];
  permissions?: InputMaybe<Scalars['Int']['input']>;
  username: Scalars['String']['input'];
};


export type MutationUpdateWatchProgressArgs = {
  fileId: Scalars['Int']['input'];
  progressPercent: Scalars['Float']['input'];
  userId?: InputMaybe<Scalars['String']['input']>;
};

export enum OrderBy {
  AddedAt = 'ADDED_AT',
  Alphabetical = 'ALPHABETICAL',
  Rating = 'RATING',
  ReleasedAt = 'RELEASED_AT',
  SeasonEpisode = 'SEASON_EPISODE'
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
  item: ItemNode;
  itemList: ItemNodeConnection;
  libraries: Array<Library>;
  library: Library;
  /** Used during library setup to pick the library path */
  listFiles: Array<Scalars['String']['output']>;
  root: RootNode;
  rootList: RootNodeConnection;
  search: SearchResults;
  season: SeasonNode;
};


export type QueryItemArgs = {
  itemId: Scalars['String']['input'];
};


export type QueryItemListArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  filter: ItemNodeFilter;
  first?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryLibraryArgs = {
  libraryId: Scalars['Int']['input'];
};


export type QueryListFilesArgs = {
  path: Scalars['String']['input'];
};


export type QueryRootArgs = {
  rootId: Scalars['String']['input'];
};


export type QueryRootListArgs = {
  after?: InputMaybe<Scalars['String']['input']>;
  filter: RootNodeFilter;
  first?: InputMaybe<Scalars['Int']['input']>;
};


export type QuerySearchArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  query: Scalars['String']['input'];
};


export type QuerySeasonArgs = {
  seasonId: Scalars['String']['input'];
};

export type RootChild = ItemNode | SeasonNode;

export enum RootKind {
  Movie = 'MOVIE',
  Series = 'SERIES'
}

export type RootNode = {
  __typename: 'RootNode';
  children: Array<RootChild>;
  createdAt: Scalars['Int']['output'];
  episodeCount: Scalars['Int']['output'];
  files: Array<ItemNode>;
  id: Scalars['String']['output'];
  kind: RootKind;
  lastAddedAt: Scalars['Int']['output'];
  libraryId: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  nextItem: Maybe<ItemNode>;
  properties: RootNodeProperties;
  seasonCount: Scalars['Int']['output'];
  seasons: Array<SeasonNode>;
  unplayedItems: Scalars['Int']['output'];
  updatedAt: Scalars['Int']['output'];
};

export type RootNodeConnection = {
  __typename: 'RootNodeConnection';
  /** A list of edges. */
  edges: Array<RootNodeEdge>;
  /** A list of nodes. */
  nodes: Array<RootNode>;
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
};

/** An edge in a connection. */
export type RootNodeEdge = {
  __typename: 'RootNodeEdge';
  /** A cursor for use in pagination */
  cursor: Scalars['String']['output'];
  /** The item at the end of the edge */
  node: RootNode;
};

export type RootNodeFilter = {
  kinds?: InputMaybe<Array<RootKind>>;
  libraryId?: InputMaybe<Scalars['Int']['input']>;
  orderBy?: InputMaybe<OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  watched?: InputMaybe<Scalars['Boolean']['input']>;
};

export type RootNodeProperties = {
  __typename: 'RootNodeProperties';
  backgroundImage: Maybe<Asset>;
  createdAt: Maybe<Scalars['Int']['output']>;
  description: Maybe<Scalars['String']['output']>;
  endedAt: Maybe<Scalars['Int']['output']>;
  posterImage: Maybe<Asset>;
  rating: Maybe<Scalars['Float']['output']>;
  releasedAt: Maybe<Scalars['Int']['output']>;
  runtimeMinutes: Maybe<Scalars['Int']['output']>;
  thumbnailImage: Maybe<Asset>;
  updatedAt: Maybe<Scalars['Int']['output']>;
};

export type SearchResults = {
  __typename: 'SearchResults';
  items: Array<ItemNode>;
  roots: Array<RootNode>;
};

export type SeasonNode = {
  __typename: 'SeasonNode';
  createdAt: Scalars['Int']['output'];
  episodeCount: Scalars['Int']['output'];
  files: Array<ItemNode>;
  id: Scalars['String']['output'];
  lastAddedAt: Scalars['Int']['output'];
  name: Scalars['String']['output'];
  nextItem: Maybe<ItemNode>;
  order: Scalars['Int']['output'];
  properties: SeasonNodeProperties;
  rootId: Scalars['String']['output'];
  seasonNumber: Scalars['Int']['output'];
  unplayedItems: Scalars['Int']['output'];
  updatedAt: Scalars['Int']['output'];
};

export type SeasonNodeProperties = {
  __typename: 'SeasonNodeProperties';
  backgroundImage: Maybe<Asset>;
  createdAt: Maybe<Scalars['Int']['output']>;
  description: Maybe<Scalars['String']['output']>;
  endedAt: Maybe<Scalars['Int']['output']>;
  posterImage: Maybe<Asset>;
  rating: Maybe<Scalars['Float']['output']>;
  releasedAt: Maybe<Scalars['Int']['output']>;
  runtimeMinutes: Maybe<Scalars['Int']['output']>;
  seasonNumber: Maybe<Scalars['Int']['output']>;
  thumbnailImage: Maybe<Asset>;
  updatedAt: Maybe<Scalars['Int']['output']>;
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
  defaultAudioIso6391: Maybe<Scalars['String']['output']>;
  defaultSubtitleIso6391: Maybe<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  permissions: Scalars['Int']['output'];
  subtitlesEnabled: Scalars['Boolean']['output'];
  username: Scalars['String']['output'];
};

export type WatchProgress = {
  __typename: 'WatchProgress';
  completed: Scalars['Boolean']['output'];
  createdAt: Scalars['Int']['output'];
  fileId: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  itemId: Scalars['String']['output'];
  progressPercent: Scalars['Float']['output'];
  updatedAt: Scalars['Int']['output'];
  userId: Scalars['String']['output'];
};

export type GetActivitiesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetActivitiesQuery = { activities: Array<{ __typename: 'Activity', taskType: string, title: string, current: number, total: number, progressPercent: number }> };

export type GetFilesQueryVariables = Exact<{
  path: Scalars['String']['input'];
}>;


export type GetFilesQuery = { listFiles: Array<string> };

export type EpisodeCardFragment = (
  { __typename: 'ItemNode', id: string, name: string, properties: { __typename: 'ItemNodeProperties', description: string | null, seasonNumber: number | null, episodeNumber: number | null, releasedAt: number | null, runtimeMinutes: number | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null }
  & { ' $fragmentRefs'?: { 'GetPathForItemFragment': GetPathForItemFragment } }
) & { ' $fragmentName'?: 'EpisodeCardFragment' };

export type ImageAssetFragment = { __typename: 'Asset', id: number, thumbhash: string | null } & { ' $fragmentName'?: 'ImageAssetFragment' };

export type RunImportWatchStatesMutationVariables = Exact<{
  input: ImportWatchStatesInput;
}>;


export type RunImportWatchStatesMutation = { importWatchStates: { __typename: 'ImportWatchStatesResult', dryRun: boolean, totalRows: number, matchedRows: number, unmatchedRows: number, conflictRows: number, willInsert: number, willOverwrite: number, imported: number, skipped: number, conflicts: Array<{ __typename: 'ImportWatchStateConflict', rowIndex: number, sourceItemId: string | null, title: string | null, itemId: string, existingProgressPercent: number, importedProgressPercent: number, reason: string }>, unmatched: Array<{ __typename: 'ImportWatchStateUnmatched', rowIndex: number, sourceItemId: string | null, title: string | null, reason: string, ambiguous: boolean }> } };

export type MediaListFragment = (
  { __typename: 'RootNode', id: string }
  & { ' $fragmentRefs'?: { 'MediaPosterFragment': MediaPosterFragment } }
) & { ' $fragmentName'?: 'MediaListFragment' };

export type MediaPosterFragment = (
  { __typename: 'RootNode', id: string, name: string, kind: RootKind, unplayedItems: number, seasonCount: number, episodeCount: number, properties: { __typename: 'RootNodeProperties', releasedAt: number | null, endedAt: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, nextItem: { __typename: 'ItemNode', id: string, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForRootFragment': GetPathForRootFragment } }
) & { ' $fragmentName'?: 'MediaPosterFragment' };

export type UpdateWatchStateMutationVariables = Exact<{
  fileId: Scalars['Int']['input'];
  progressPercent: Scalars['Float']['input'];
}>;


export type UpdateWatchStateMutation = { updateWatchProgress: Array<{ __typename: 'WatchProgress', progressPercent: number, updatedAt: number }> };

export type ItemPlaybackQueryVariables = Exact<{
  itemId: Scalars['String']['input'];
}>;


export type ItemPlaybackQuery = { item: { __typename: 'ItemNode', id: string, kind: ItemKind, name: string, rootId: string, seasonId: string | null, properties: { __typename: 'ItemNodeProperties', seasonNumber: number | null, episodeNumber: number | null, runtimeMinutes: number | null }, parent: { __typename: 'RootNode', name: string, libraryId: number } | null, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null, file: { __typename: 'File', id: number, segments: Array<{ __typename: 'FileSegment', kind: FileSegmentKind, startMs: number, endMs: number }>, timelinePreview: Array<{ __typename: 'TimelinePreviewSheet', positionMs: number, endMs: number, sheetIntervalMs: number, sheetGapSize: number, asset: { __typename: 'Asset', id: number, width: number | null, height: number | null } }> } | null, previousItem: { __typename: 'ItemNode', id: string } | null, nextItem: { __typename: 'ItemNode', id: string } | null } };

export type SearchRootResultFragment = (
  { __typename: 'RootNode', id: string, name: string, kind: RootKind, seasonCount: number, episodeCount: number, properties: { __typename: 'RootNodeProperties', releasedAt: number | null, endedAt: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null } }
  & { ' $fragmentRefs'?: { 'GetPathForRootFragment': GetPathForRootFragment } }
) & { ' $fragmentName'?: 'SearchRootResultFragment' };

export type SearchItemResultFragment = (
  { __typename: 'ItemNode', id: string, name: string, kind: ItemKind, parent: { __typename: 'RootNode', name: string, libraryId: number } | null, properties: { __typename: 'ItemNodeProperties', description: string | null, seasonNumber: number | null, episodeNumber: number | null, releasedAt: number | null, runtimeMinutes: number | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null } }
  & { ' $fragmentRefs'?: { 'GetPathForItemFragment': GetPathForItemFragment } }
) & { ' $fragmentName'?: 'SearchItemResultFragment' };

export type SearchMediaQueryVariables = Exact<{
  query: Scalars['String']['input'];
  limit?: InputMaybe<Scalars['Int']['input']>;
}>;


export type SearchMediaQuery = { search: { __typename: 'SearchResults', roots: Array<(
      { __typename: 'RootNode' }
      & { ' $fragmentRefs'?: { 'SearchRootResultFragment': SearchRootResultFragment } }
    )>, items: Array<(
      { __typename: 'ItemNode' }
      & { ' $fragmentRefs'?: { 'SearchItemResultFragment': SearchItemResultFragment } }
    )> } };

export type SeasonCardFragment = { __typename: 'SeasonNode', id: string, name: string, seasonNumber: number, order: number, unplayedItems: number, episodeCount: number, properties: { __typename: 'SeasonNodeProperties', releasedAt: number | null, endedAt: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, nextItem: { __typename: 'ItemNode', id: string, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null } | null } & { ' $fragmentName'?: 'SeasonCardFragment' };

export type LibrariesQueryVariables = Exact<{ [key: string]: never; }>;


export type LibrariesQuery = { libraries: Array<{ __typename: 'Library', id: number, name: string, createdAt: number }> };

export type GetPathForRootFragment = { __typename: 'RootNode', id: string, libraryId: number } & { ' $fragmentName'?: 'GetPathForRootFragment' };

export type GetPathForItemFragment = { __typename: 'ItemNode', kind: ItemKind, rootId: string, seasonId: string | null, parent: { __typename: 'RootNode', libraryId: number } | null } & { ' $fragmentName'?: 'GetPathForItemFragment' };

export type GetAllMediaQueryVariables = Exact<{
  filter: RootNodeFilter;
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetAllMediaQuery = { rootList: { __typename: 'RootNodeConnection', edges: Array<{ __typename: 'RootNodeEdge', node: (
        { __typename: 'RootNode' }
        & { ' $fragmentRefs'?: { 'MediaListFragment': MediaListFragment } }
      ) }>, pageInfo: { __typename: 'PageInfo', endCursor: string | null, hasNextPage: boolean } } };

export type GetLibraryMediaQueryVariables = Exact<{
  libraryId: Scalars['Int']['input'];
  filter: RootNodeFilter;
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetLibraryMediaQuery = { rootList: { __typename: 'RootNodeConnection', edges: Array<{ __typename: 'RootNodeEdge', node: (
        { __typename: 'RootNode', id: string }
        & { ' $fragmentRefs'?: { 'MediaListFragment': MediaListFragment } }
      ) }>, pageInfo: { __typename: 'PageInfo', endCursor: string | null, hasNextPage: boolean } }, library: { __typename: 'Library', id: number, name: string } };

export type GetRootByIdQueryVariables = Exact<{
  rootId: Scalars['String']['input'];
}>;


export type GetRootByIdQuery = { root: { __typename: 'RootNode', id: string, kind: RootKind, name: string, libraryId: number, unplayedItems: number, seasons: Array<(
      { __typename: 'SeasonNode', id: string, order: number }
      & { ' $fragmentRefs'?: { 'SeasonCardFragment': SeasonCardFragment } }
    )>, properties: { __typename: 'RootNodeProperties', releasedAt: number | null, endedAt: number | null, runtimeMinutes: number | null, description: string | null, posterImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, backgroundImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null }, nextItem: { __typename: 'ItemNode', id: string, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null } | null } };

export type GetRootAndSeasonQueryVariables = Exact<{
  rootId: Scalars['String']['input'];
  seasonId: Scalars['String']['input'];
}>;


export type GetRootAndSeasonQuery = { root: { __typename: 'RootNode', id: string, libraryId: number, name: string, properties: { __typename: 'RootNodeProperties', backgroundImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, posterImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null } }, season: { __typename: 'SeasonNode', id: string, name: string, seasonNumber: number, unplayedItems: number, properties: { __typename: 'SeasonNodeProperties', releasedAt: number | null, endedAt: number | null, runtimeMinutes: number | null, description: string | null, posterImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, thumbnailImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, backgroundImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null }, nextItem: { __typename: 'ItemNode', id: string, watchProgress: { __typename: 'WatchProgress', progressPercent: number, completed: boolean, updatedAt: number } | null } | null } };

export type GetSeasonEpisodesQueryVariables = Exact<{
  filter: ItemNodeFilter;
  after?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetSeasonEpisodesQuery = { itemList: { __typename: 'ItemNodeConnection', edges: Array<{ __typename: 'ItemNodeEdge', node: (
        { __typename: 'ItemNode', id: string }
        & { ' $fragmentRefs'?: { 'EpisodeCardFragment': EpisodeCardFragment } }
      ) }>, pageInfo: { __typename: 'PageInfo', endCursor: string | null, hasNextPage: boolean } } };

export type SignupMutationVariables = Exact<{
  username: Scalars['String']['input'];
  password: Scalars['String']['input'];
}>;


export type SignupMutation = { signup: { __typename: 'User', id: string, username: string } };

export type GetLibrariesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetLibrariesQuery = { libraries: Array<{ __typename: 'Library', id: number, name: string, path: string }> };

export type CreateLibraryMutationVariables = Exact<{
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
}>;


export type CreateLibraryMutation = { createLibrary: { __typename: 'Library', id: number, name: string, path: string } };

export const ImageAssetFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}}]} as unknown as DocumentNode<ImageAssetFragment, unknown>;
export const GetPathForItemFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForItem"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"rootId"}},{"kind":"Field","name":{"kind":"Name","value":"seasonId"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]}}]} as unknown as DocumentNode<GetPathForItemFragment, unknown>;
export const EpisodeCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForItem"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForItem"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"rootId"}},{"kind":"Field","name":{"kind":"Name","value":"seasonId"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]}}]} as unknown as DocumentNode<EpisodeCardFragment, unknown>;
export const GetPathForRootFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForRoot"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<GetPathForRootFragment, unknown>;
export const MediaPosterFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MediaPoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForRoot"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForRoot"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<MediaPosterFragment, unknown>;
export const MediaListFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MediaList"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"MediaPoster"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForRoot"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MediaPoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForRoot"}}]}}]} as unknown as DocumentNode<MediaListFragment, unknown>;
export const SearchRootResultFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchRootResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForRoot"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForRoot"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<SearchRootResultFragment, unknown>;
export const SearchItemResultFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchItemResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForItem"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForItem"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"rootId"}},{"kind":"Field","name":{"kind":"Name","value":"seasonId"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]}}]} as unknown as DocumentNode<SearchItemResultFragment, unknown>;
export const SeasonCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SeasonCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SeasonNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"order"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}}]} as unknown as DocumentNode<SeasonCardFragment, unknown>;
export const GetActivitiesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetActivities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskType"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"current"}},{"kind":"Field","name":{"kind":"Name","value":"total"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}}]}}]}}]} as unknown as DocumentNode<GetActivitiesQuery, GetActivitiesQueryVariables>;
export const GetFilesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetFiles"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"listFiles"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}]}]}}]} as unknown as DocumentNode<GetFilesQuery, GetFilesQueryVariables>;
export const RunImportWatchStatesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"RunImportWatchStates"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"input"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ImportWatchStatesInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"importWatchStates"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"input"},"value":{"kind":"Variable","name":{"kind":"Name","value":"input"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"dryRun"}},{"kind":"Field","name":{"kind":"Name","value":"totalRows"}},{"kind":"Field","name":{"kind":"Name","value":"matchedRows"}},{"kind":"Field","name":{"kind":"Name","value":"unmatchedRows"}},{"kind":"Field","name":{"kind":"Name","value":"conflictRows"}},{"kind":"Field","name":{"kind":"Name","value":"willInsert"}},{"kind":"Field","name":{"kind":"Name","value":"willOverwrite"}},{"kind":"Field","name":{"kind":"Name","value":"imported"}},{"kind":"Field","name":{"kind":"Name","value":"skipped"}},{"kind":"Field","name":{"kind":"Name","value":"conflicts"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rowIndex"}},{"kind":"Field","name":{"kind":"Name","value":"sourceItemId"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"itemId"}},{"kind":"Field","name":{"kind":"Name","value":"existingProgressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"importedProgressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"reason"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unmatched"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rowIndex"}},{"kind":"Field","name":{"kind":"Name","value":"sourceItemId"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"reason"}},{"kind":"Field","name":{"kind":"Name","value":"ambiguous"}}]}}]}}]}}]} as unknown as DocumentNode<RunImportWatchStatesMutation, RunImportWatchStatesMutationVariables>;
export const UpdateWatchStateDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateWatchState"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"fileId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"progressPercent"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Float"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateWatchProgress"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"fileId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"fileId"}}},{"kind":"Argument","name":{"kind":"Name","value":"progressPercent"},"value":{"kind":"Variable","name":{"kind":"Name","value":"progressPercent"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}}]} as unknown as DocumentNode<UpdateWatchStateMutation, UpdateWatchStateMutationVariables>;
export const ItemPlaybackDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ItemPlayback"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"item"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"itemId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"itemId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"rootId"}},{"kind":"Field","name":{"kind":"Name","value":"seasonId"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"file"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"segments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"startMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}}]}},{"kind":"Field","name":{"kind":"Name","value":"timelinePreview"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"previousItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<ItemPlaybackQuery, ItemPlaybackQueryVariables>;
export const SearchMediaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SearchMedia"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"query"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"limit"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"search"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"query"},"value":{"kind":"Variable","name":{"kind":"Name","value":"query"}}},{"kind":"Argument","name":{"kind":"Name","value":"limit"},"value":{"kind":"Variable","name":{"kind":"Name","value":"limit"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"roots"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SearchRootResult"}}]}},{"kind":"Field","name":{"kind":"Name","value":"items"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SearchItemResult"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForRoot"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForItem"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"rootId"}},{"kind":"Field","name":{"kind":"Name","value":"seasonId"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchRootResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForRoot"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchItemResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForItem"}}]}}]} as unknown as DocumentNode<SearchMediaQuery, SearchMediaQueryVariables>;
export const LibrariesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}}]}}]}}]} as unknown as DocumentNode<LibrariesQuery, LibrariesQueryVariables>;
export const GetAllMediaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetAllMedia"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RootNodeFilter"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rootList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}},{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"45"}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"MediaList"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForRoot"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MediaPoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForRoot"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MediaList"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"MediaPoster"}}]}}]} as unknown as DocumentNode<GetAllMediaQuery, GetAllMediaQueryVariables>;
export const GetLibraryMediaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLibraryMedia"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"RootNodeFilter"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rootList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}},{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"45"}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"MediaList"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"library"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForRoot"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MediaPoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForRoot"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"MediaList"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"RootNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"MediaPoster"}}]}}]} as unknown as DocumentNode<GetLibraryMediaQuery, GetLibraryMediaQueryVariables>;
export const GetRootByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetRootById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"rootId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"root"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"rootId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"rootId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"seasons"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"order"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"SeasonCard"}}]}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"backgroundImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SeasonCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"SeasonNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"order"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}}]}}]} as unknown as DocumentNode<GetRootByIdQuery, GetRootByIdQueryVariables>;
export const GetRootAndSeasonDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetRootAndSeason"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"rootId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"seasonId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"root"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"rootId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"rootId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"backgroundImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"season"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"seasonId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"seasonId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"backgroundImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"endedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"description"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextItem"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedItems"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}}]} as unknown as DocumentNode<GetRootAndSeasonQuery, GetRootAndSeasonQueryVariables>;
export const GetSeasonEpisodesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSeasonEpisodes"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNodeFilter"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"itemList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}},{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"EpisodeCard"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForItem"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"rootId"}},{"kind":"Field","name":{"kind":"Name","value":"seasonId"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"ItemNode"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"releasedAt"}},{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgress"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}},{"kind":"Field","name":{"kind":"Name","value":"completed"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForItem"}}]}}]} as unknown as DocumentNode<GetSeasonEpisodesQuery, GetSeasonEpisodesQueryVariables>;
export const SignupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Signup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"password"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"signup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"password"},"value":{"kind":"Variable","name":{"kind":"Name","value":"password"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}}]}}]} as unknown as DocumentNode<SignupMutation, SignupMutationVariables>;
export const GetLibrariesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLibraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}}]}}]}}]} as unknown as DocumentNode<GetLibrariesQuery, GetLibrariesQueryVariables>;
export const CreateLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}}]}}]}}]} as unknown as DocumentNode<CreateLibraryMutation, CreateLibraryMutationVariables>;