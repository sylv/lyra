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
  aspectRatio: Maybe<Scalars['String']['output']>;
  contentEncoding: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['Int']['output'];
  hashSha256: Maybe<Scalars['String']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  id: Scalars['String']['output'];
  mimeType: Maybe<Scalars['String']['output']>;
  signedUrl: Scalars['String']['output'];
  sizeBytes: Maybe<Scalars['Int']['output']>;
  sourceUrl: Maybe<Scalars['String']['output']>;
  thumbhash: Maybe<Scalars['String']['output']>;
  uncompressedSizeBytes: Maybe<Scalars['Int']['output']>;
  width: Maybe<Scalars['Int']['output']>;
};

export type ContentRating = {
  __typename: 'ContentRating';
  countryCode: Scalars['String']['output'];
  rating: Scalars['String']['output'];
};

export type DisabledSubtitlesHintInput = {
  fileId: Scalars['String']['input'];
  sourceTrackId: Scalars['String']['input'];
};

export type File = {
  __typename: 'File';
  discoveredAt: Scalars['Int']['output'];
  editionName: Maybe<Scalars['String']['output']>;
  height: Maybe<Scalars['Int']['output']>;
  id: Scalars['String']['output'];
  libraryId: Scalars['String']['output'];
  playback: Playback;
  probe: Maybe<FileProbe>;
  relativePath: Scalars['String']['output'];
  resumeHint: Maybe<ResumeHint>;
  scannedAt: Maybe<Scalars['Int']['output']>;
  segments: Array<FileSegment>;
  sizeBytes: Scalars['Int']['output'];
  subtitlesExtractedAt: Maybe<Scalars['Int']['output']>;
  timelinePreview: Array<TimelinePreviewSheet>;
  unavailableAt: Maybe<Scalars['Int']['output']>;
  width: Maybe<Scalars['Int']['output']>;
};


export type FilePlaybackArgs = {
  languageHint?: InputMaybe<Scalars['String']['input']>;
};

export type FileProbe = {
  __typename: 'FileProbe';
  audioBitrate: Maybe<Scalars['Int']['output']>;
  audioChannels: Maybe<Scalars['Int']['output']>;
  audioCodec: Maybe<Scalars['String']['output']>;
  durationSeconds: Maybe<Scalars['Int']['output']>;
  fps: Maybe<Scalars['Float']['output']>;
  hasSubtitles: Scalars['Boolean']['output'];
  height: Maybe<Scalars['Int']['output']>;
  runtimeMinutes: Maybe<Scalars['Int']['output']>;
  videoBitrate: Maybe<Scalars['Int']['output']>;
  videoCodec: Maybe<Scalars['String']['output']>;
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

export enum MetadataStatus {
  Airing = 'AIRING',
  Cancelled = 'CANCELLED',
  Finished = 'FINISHED',
  InTheaters = 'IN_THEATERS',
  Released = 'RELEASED',
  Returning = 'RETURNING',
  Upcoming = 'UPCOMING'
}

export type Mutation = {
  __typename: 'Mutation';
  createLibrary: Library;
  createUserInvite: User;
  deleteLibrary: Scalars['Boolean']['output'];
  deleteUser: Scalars['Boolean']['output'];
  disabledSubtitlesHint: Scalars['Boolean']['output'];
  resetUserInvite: User;
  setPreferredAudio: User;
  signup: User;
  updateLibrary: Library;
  updateUser: User;
  updateWatchProgress: Array<WatchProgress>;
};


export type MutationCreateLibraryArgs = {
  name: Scalars['String']['input'];
  path: Scalars['String']['input'];
  pinned?: InputMaybe<Scalars['Boolean']['input']>;
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


export type MutationDisabledSubtitlesHintArgs = {
  input: DisabledSubtitlesHintInput;
};


export type MutationResetUserInviteArgs = {
  userId: Scalars['String']['input'];
};


export type MutationSetPreferredAudioArgs = {
  disposition?: InputMaybe<TrackDispositionPreference>;
  language?: InputMaybe<Scalars['String']['input']>;
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
  pinned: Scalars['Boolean']['input'];
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
  currentPlayable: Maybe<Node>;
  defaultFile: Maybe<File>;
  episodeCount: Scalars['Int']['output'];
  episodeNumber: Maybe<Scalars['Int']['output']>;
  id: Scalars['String']['output'];
  kind: NodeKind;
  lastAddedAt: Scalars['Int']['output'];
  lastFingerprintVersion: Maybe<Scalars['Int']['output']>;
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
  unavailableAt: Maybe<Scalars['Int']['output']>;
  unplayedCount: Maybe<Scalars['Int']['output']>;
  updatedAt: Scalars['Int']['output'];
  watchProgressHint: Maybe<Scalars['Float']['output']>;
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
  releasedAfter?: InputMaybe<Scalars['Int']['input']>;
  rootId?: InputMaybe<Scalars['String']['input']>;
  searchTerm?: InputMaybe<Scalars['String']['input']>;
  seasonNumbers?: InputMaybe<Array<Scalars['Int']['input']>>;
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
  backdropImage: Maybe<Asset>;
  contentRating: Maybe<ContentRating>;
  createdAt: Maybe<Scalars['Int']['output']>;
  description: Maybe<Scalars['String']['output']>;
  displayDetail: Maybe<Scalars['String']['output']>;
  displayName: Scalars['String']['output'];
  episodeNumber: Maybe<Scalars['Int']['output']>;
  firstAired: Maybe<Scalars['Int']['output']>;
  lastAired: Maybe<Scalars['Int']['output']>;
  logoImage: Maybe<Asset>;
  posterImage: Maybe<Asset>;
  rating: Maybe<Scalars['Float']['output']>;
  seasonNumber: Maybe<Scalars['Int']['output']>;
  status: Maybe<MetadataStatus>;
  tagline: Maybe<Scalars['String']['output']>;
  thumbnailImage: Maybe<Asset>;
  updatedAt: Maybe<Scalars['Int']['output']>;
};

export enum OrderBy {
  AddedAt = 'ADDED_AT',
  Alphabetical = 'ALPHABETICAL',
  FirstAired = 'FIRST_AIRED',
  LastAddedAt = 'LAST_ADDED_AT',
  LastAired = 'LAST_AIRED',
  Order = 'ORDER',
  Rating = 'RATING',
  ReleasedAt = 'RELEASED_AT',
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

export type Playback = {
  __typename: 'Playback';
  audio: Array<PlaybackAudioTrack>;
  hlsUrlTemplate: Scalars['String']['output'];
  subtitles: Array<PlaybackSubtitleTrack>;
  video: Array<PlaybackVideoTrack>;
};

export enum PlaybackAudioCodec {
  Aac = 'AAC'
}

export enum PlaybackAudioProfileId {
  Aac = 'AAC'
}

export type PlaybackAudioRendition = {
  __typename: 'PlaybackAudioRendition';
  codec: PlaybackAudioCodec;
  codecTag: Scalars['String']['output'];
  displayInfo: Scalars['String']['output'];
  pairId: Scalars['String']['output'];
  profileId: PlaybackAudioProfileId;
};

export type PlaybackAudioTrack = {
  __typename: 'PlaybackAudioTrack';
  autoselect: Scalars['Boolean']['output'];
  displayName: Scalars['String']['output'];
  languageBcp47: Maybe<Scalars['String']['output']>;
  renditions: Array<PlaybackAudioRendition>;
  sourceTrackId: Scalars['String']['output'];
};

export enum PlaybackSubtitleCodec {
  Ass = 'ASS',
  MovText = 'MOV_TEXT',
  Pgs = 'PGS',
  Srt = 'SRT',
  Text = 'TEXT',
  Ttml = 'TTML',
  VobSub = 'VOB_SUB',
  Vtt = 'VTT'
}

export enum PlaybackSubtitleKind {
  Captions = 'CAPTIONS',
  Commentary = 'COMMENTARY',
  Forced = 'FORCED',
  Subtitles = 'SUBTITLES'
}

export type PlaybackSubtitleRendition = {
  __typename: 'PlaybackSubtitleRendition';
  codec: PlaybackSubtitleCodec;
  displayInfo: Scalars['String']['output'];
  signedUrl: Scalars['String']['output'];
  variantId: Maybe<Scalars['String']['output']>;
};

export type PlaybackSubtitleTrack = {
  __typename: 'PlaybackSubtitleTrack';
  autoselect: Scalars['Boolean']['output'];
  displayName: Scalars['String']['output'];
  kind: PlaybackSubtitleKind;
  languageBcp47: Maybe<Scalars['String']['output']>;
  renditions: Array<PlaybackSubtitleRendition>;
  sourceTrackId: Scalars['String']['output'];
};

export enum PlaybackVideoCodec {
  Av_1 = 'AV_1',
  H264 = 'H264',
  H265 = 'H265'
}

export enum PlaybackVideoProfileId {
  Copy = 'COPY',
  H264 = 'H264'
}

export type PlaybackVideoRendition = {
  __typename: 'PlaybackVideoRendition';
  codec: PlaybackVideoCodec;
  codecTag: Scalars['String']['output'];
  displayInfo: Scalars['String']['output'];
  pairId: Scalars['String']['output'];
  profileId: PlaybackVideoProfileId;
};

export type PlaybackVideoTrack = {
  __typename: 'PlaybackVideoTrack';
  autoselect: Scalars['Boolean']['output'];
  displayName: Scalars['String']['output'];
  renditions: Array<PlaybackVideoRendition>;
  sourceTrackId: Scalars['String']['output'];
};

export type Query = {
  __typename: 'Query';
  activities: Array<Activity>;
  libraries: Array<Library>;
  library: Library;
  listFiles: Array<Scalars['String']['output']>;
  node: Node;
  nodeList: NodeConnection;
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

export type ResumeHint = {
  __typename: 'ResumeHint';
  id: Scalars['String']['output'];
  startMs: Scalars['Int']['output'];
  updatedAt: Scalars['Int']['output'];
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

export type User = {
  __typename: 'User';
  createdAt: Scalars['Int']['output'];
  id: Scalars['String']['output'];
  inviteCode: Maybe<Scalars['String']['output']>;
  lastSeenAt: Maybe<Scalars['Int']['output']>;
  passwordHash: Maybe<Scalars['String']['output']>;
  permissions: Scalars['Int']['output'];
  preferredAudioDisposition: Maybe<Scalars['String']['output']>;
  preferredAudioLanguage: Maybe<Scalars['String']['output']>;
  preferredSubtitleLanguages: Scalars['String']['output'];
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
  tmdbId: Maybe<Scalars['Int']['output']>;
  updatedAt: Scalars['Int']['output'];
  userId: Scalars['String']['output'];
};

export type GetActivitiesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetActivitiesQuery = { activities: Array<{ __typename: 'Activity', taskType: string, title: string, current: number | null, total: number | null, progressPercent: number | null }> };

export type GetFilesQueryVariables = Exact<{
  path: Scalars['String']['input'];
}>;


export type GetFilesQuery = { listFiles: Array<string> };

export type ImageAssetFragment = { __typename: 'Asset', id: string, signedUrl: string, thumbhash: string | null } & { ' $fragmentName'?: 'ImageAssetFragment' };

export type EpisodeCardFragment = (
  { __typename: 'Node', id: string, unavailableAt: number | null, watchProgressHint: number | null, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, firstAired: number | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, defaultFile: { __typename: 'File', id: string, probe: { __typename: 'FileProbe', runtimeMinutes: number | null } | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'EpisodeCardFragment' };

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
    ) | null }, currentPlayable: { __typename: 'Node', id: string, watchProgressHint: number | null } | null }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'NodePosterFragment' };

export type PlayerQueryVariables = Exact<{
  nodeId: Scalars['String']['input'];
}>;


export type PlayerQuery = { node: (
    { __typename: 'Node', id: string, properties: { __typename: 'NodeProperties', displayName: string, seasonNumber: number | null, episodeNumber: number | null }, root: { __typename: 'Node', properties: { __typename: 'NodeProperties', displayName: string } } | null, defaultFile: { __typename: 'File', id: string, height: number | null, width: number | null, resumeHint: { __typename: 'ResumeHint', startMs: number, updatedAt: number } | null, probe: { __typename: 'FileProbe', durationSeconds: number | null, width: number | null, height: number | null } | null, playback: { __typename: 'Playback', hlsUrlTemplate: string, video: Array<(
          { __typename: 'PlaybackVideoTrack' }
          & { ' $fragmentRefs'?: { 'PlayerVideoTrackFragment': PlayerVideoTrackFragment } }
        )>, audio: Array<(
          { __typename: 'PlaybackAudioTrack' }
          & { ' $fragmentRefs'?: { 'PlayerAudioTrackFragment': PlayerAudioTrackFragment } }
        )>, subtitles: Array<(
          { __typename: 'PlaybackSubtitleTrack' }
          & { ' $fragmentRefs'?: { 'PlayerSubtitleTrackFragment': PlayerSubtitleTrackFragment } }
        )> }, timelinePreview: Array<(
        { __typename: 'TimelinePreviewSheet' }
        & { ' $fragmentRefs'?: { 'PlayerTimelinePreviewSheetFragment': PlayerTimelinePreviewSheetFragment } }
      )>, segments: Array<{ __typename: 'FileSegment', kind: FileSegmentKind, startMs: number, endMs: number }> } | null, previousPlayable: (
      { __typename: 'Node', id: string }
      & { ' $fragmentRefs'?: { 'PlayerItemCardFragment': PlayerItemCardFragment } }
    ) | null, nextPlayable: (
      { __typename: 'Node', id: string }
      & { ' $fragmentRefs'?: { 'PlayerItemCardFragment': PlayerItemCardFragment } }
    ) | null }
    & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
  ) };

export type PlayerAudioTrackFragment = { __typename: 'PlaybackAudioTrack', sourceTrackId: string, displayName: string, autoselect: boolean, languageBcp47: string | null, renditions: Array<{ __typename: 'PlaybackAudioRendition', pairId: string, profileId: PlaybackAudioProfileId, codec: PlaybackAudioCodec, displayInfo: string, codecTag: string }> } & { ' $fragmentName'?: 'PlayerAudioTrackFragment' };

export type PlayerVideoTrackFragment = { __typename: 'PlaybackVideoTrack', sourceTrackId: string, displayName: string, autoselect: boolean, renditions: Array<{ __typename: 'PlaybackVideoRendition', pairId: string, profileId: PlaybackVideoProfileId, codec: PlaybackVideoCodec, displayInfo: string, codecTag: string }> } & { ' $fragmentName'?: 'PlayerVideoTrackFragment' };

export type PlayerSubtitleTrackFragment = { __typename: 'PlaybackSubtitleTrack', sourceTrackId: string, displayName: string, autoselect: boolean, kind: PlaybackSubtitleKind, languageBcp47: string | null, renditions: Array<{ __typename: 'PlaybackSubtitleRendition', variantId: string | null, codec: PlaybackSubtitleCodec, displayInfo: string, signedUrl: string }> } & { ' $fragmentName'?: 'PlayerSubtitleTrackFragment' };

export type PlayerTimelinePreviewSheetFragment = { __typename: 'TimelinePreviewSheet', positionMs: number, endMs: number, sheetIntervalMs: number, sheetGapSize: number, asset: { __typename: 'Asset', id: string, signedUrl: string, width: number | null, height: number | null } } & { ' $fragmentName'?: 'PlayerTimelinePreviewSheetFragment' };

export type PlayerItemCardFragment = (
  { __typename: 'Node', id: string, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null } }
  & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
) & { ' $fragmentName'?: 'PlayerItemCardFragment' };

export type SearchNodeResultFragment = (
  { __typename: 'Node', id: string, kind: NodeKind, libraryId: string, seasonCount: number, episodeCount: number, root: { __typename: 'Node', properties: { __typename: 'NodeProperties', displayName: string } } | null, properties: { __typename: 'NodeProperties', displayName: string, description: string | null, seasonNumber: number | null, episodeNumber: number | null, firstAired: number | null, lastAired: number | null, posterImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null, thumbnailImage: (
      { __typename: 'Asset' }
      & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
    ) | null }, defaultFile: { __typename: 'File', id: string, probe: { __typename: 'FileProbe', runtimeMinutes: number | null } | null } | null }
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
    ) | null }, currentPlayable: { __typename: 'Node', id: string, watchProgressHint: number | null } | null }
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

export type SidebarNavigationQueryVariables = Exact<{ [key: string]: never; }>;


export type SidebarNavigationQuery = { libraries: Array<{ __typename: 'Library', id: string, name: string, createdAt: number, pinned: boolean }> };

export type SidebarViewerQueryVariables = Exact<{ [key: string]: never; }>;


export type SidebarViewerQuery = { viewer: { __typename: 'User', id: string, permissions: number } | null };

export type GetPathForNodeFragment = { __typename: 'Node', id: string, libraryId: string } & { ' $fragmentName'?: 'GetPathForNodeFragment' };

export type HomeRecentlyAddedQueryVariables = Exact<{ [key: string]: never; }>;


export type HomeRecentlyAddedQuery = { nodeList: { __typename: 'NodeConnection', nodes: Array<(
      { __typename: 'Node', id: string }
      & { ' $fragmentRefs'?: { 'NodePosterFragment': NodePosterFragment } }
    )> } };

export type HomeWatchedQueryVariables = Exact<{ [key: string]: never; }>;


export type HomeWatchedQuery = { nodeList: { __typename: 'NodeConnection', nodes: Array<(
      { __typename: 'Node', id: string }
      & { ' $fragmentRefs'?: { 'NodePosterFragment': NodePosterFragment } }
    )> } };

export type GetNodeByIdQueryVariables = Exact<{
  nodeId: Scalars['String']['input'];
}>;


export type GetNodeByIdQuery = { node: (
    { __typename: 'Node', id: string, kind: NodeKind, unavailableAt: number | null, unplayedCount: number | null, seasonCount: number, seasonNumber: number | null, episodeNumber: number | null, watchProgressHint: number | null, parent: (
      { __typename: 'Node', id: string }
      & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
    ) | null, root: (
      { __typename: 'Node', id: string, properties: { __typename: 'NodeProperties', displayName: string } }
      & { ' $fragmentRefs'?: { 'GetPathForNodeFragment': GetPathForNodeFragment } }
    ) | null, properties: { __typename: 'NodeProperties', displayName: string, tagline: string | null, description: string | null, posterImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, thumbnailImage: (
        { __typename: 'Asset' }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, logoImage: { __typename: 'Asset', id: string, signedUrl: string, aspectRatio: string | null } | null, backdropImage: (
        { __typename: 'Asset', signedUrl: string, aspectRatio: string | null }
        & { ' $fragmentRefs'?: { 'ImageAssetFragment': ImageAssetFragment } }
      ) | null, contentRating: { __typename: 'ContentRating', rating: string } | null }, currentPlayable: { __typename: 'Node', id: string, seasonNumber: number | null, episodeNumber: number | null, watchProgressHint: number | null } | null, defaultFile: { __typename: 'File', id: string, probe: { __typename: 'FileProbe', runtimeMinutes: number | null, width: number | null, height: number | null, videoCodec: string | null, videoBitrate: number | null } | null } | null }
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
export const EpisodeCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"defaultFile"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"probe"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<EpisodeCardFragment, unknown>;
export const NodePosterFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"currentPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<NodePosterFragment, unknown>;
export const PlayerAudioTrackFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerAudioTrack"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"PlaybackAudioTrack"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sourceTrackId"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"autoselect"}},{"kind":"Field","name":{"kind":"Name","value":"languageBcp47"}},{"kind":"Field","name":{"kind":"Name","value":"renditions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pairId"}},{"kind":"Field","name":{"kind":"Name","value":"profileId"}},{"kind":"Field","name":{"kind":"Name","value":"codec"}},{"kind":"Field","name":{"kind":"Name","value":"displayInfo"}},{"kind":"Field","name":{"kind":"Name","value":"codecTag"}}]}}]}}]} as unknown as DocumentNode<PlayerAudioTrackFragment, unknown>;
export const PlayerVideoTrackFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerVideoTrack"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"PlaybackVideoTrack"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sourceTrackId"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"autoselect"}},{"kind":"Field","name":{"kind":"Name","value":"renditions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pairId"}},{"kind":"Field","name":{"kind":"Name","value":"profileId"}},{"kind":"Field","name":{"kind":"Name","value":"codec"}},{"kind":"Field","name":{"kind":"Name","value":"displayInfo"}},{"kind":"Field","name":{"kind":"Name","value":"codecTag"}}]}}]}}]} as unknown as DocumentNode<PlayerVideoTrackFragment, unknown>;
export const PlayerSubtitleTrackFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerSubtitleTrack"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"PlaybackSubtitleTrack"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sourceTrackId"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"autoselect"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"languageBcp47"}},{"kind":"Field","name":{"kind":"Name","value":"renditions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"variantId"}},{"kind":"Field","name":{"kind":"Name","value":"codec"}},{"kind":"Field","name":{"kind":"Name","value":"displayInfo"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}}]}}]}}]} as unknown as DocumentNode<PlayerSubtitleTrackFragment, unknown>;
export const PlayerTimelinePreviewSheetFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}}]} as unknown as DocumentNode<PlayerTimelinePreviewSheetFragment, unknown>;
export const PlayerItemCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerItemCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}}]} as unknown as DocumentNode<PlayerItemCardFragment, unknown>;
export const SearchNodeResultFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchNodeResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"defaultFile"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"probe"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<SearchNodeResultFragment, unknown>;
export const SeasonCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SeasonCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"currentPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}}]} as unknown as DocumentNode<SeasonCardFragment, unknown>;
export const LibraryCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<LibraryCardFragment, unknown>;
export const UserCardFragmentDoc = {"kind":"Document","definitions":[{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UserCardFragment, unknown>;
export const GetActivitiesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetActivities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"activities"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"taskType"}},{"kind":"Field","name":{"kind":"Name","value":"title"}},{"kind":"Field","name":{"kind":"Name","value":"current"}},{"kind":"Field","name":{"kind":"Name","value":"total"}},{"kind":"Field","name":{"kind":"Name","value":"progressPercent"}}]}}]}}]} as unknown as DocumentNode<GetActivitiesQuery, GetActivitiesQueryVariables>;
export const GetFilesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetFiles"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"listFiles"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}}]}]}}]} as unknown as DocumentNode<GetFilesQuery, GetFilesQueryVariables>;
export const NodePageDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"NodePage"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"after"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"first"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"NodeFilter"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"after"},"value":{"kind":"Variable","name":{"kind":"Name","value":"after"}}},{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"Variable","name":{"kind":"Name","value":"first"}}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"edges"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"EpisodeCard"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"pageInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"endCursor"}},{"kind":"Field","name":{"kind":"Name","value":"hasNextPage"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"currentPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"EpisodeCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"defaultFile"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"probe"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<NodePageQuery, NodePageQueryVariables>;
export const PlayerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"Player"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"nodeId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"defaultFile"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"height"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"resumeHint"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"startMs"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}}]}},{"kind":"Field","name":{"kind":"Name","value":"probe"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"durationSeconds"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}},{"kind":"Field","name":{"kind":"Name","value":"playback"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"hlsUrlTemplate"}},{"kind":"Field","name":{"kind":"Name","value":"video"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerVideoTrack"}}]}},{"kind":"Field","name":{"kind":"Name","value":"audio"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerAudioTrack"}}]}},{"kind":"Field","name":{"kind":"Name","value":"subtitles"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerSubtitleTrack"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"timelinePreview"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"}}]}},{"kind":"Field","name":{"kind":"Name","value":"segments"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"startMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"previousPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerItemCard"}}]}},{"kind":"Field","name":{"kind":"Name","value":"nextPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"PlayerItemCard"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerVideoTrack"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"PlaybackVideoTrack"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sourceTrackId"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"autoselect"}},{"kind":"Field","name":{"kind":"Name","value":"renditions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pairId"}},{"kind":"Field","name":{"kind":"Name","value":"profileId"}},{"kind":"Field","name":{"kind":"Name","value":"codec"}},{"kind":"Field","name":{"kind":"Name","value":"displayInfo"}},{"kind":"Field","name":{"kind":"Name","value":"codecTag"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerAudioTrack"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"PlaybackAudioTrack"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sourceTrackId"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"autoselect"}},{"kind":"Field","name":{"kind":"Name","value":"languageBcp47"}},{"kind":"Field","name":{"kind":"Name","value":"renditions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"pairId"}},{"kind":"Field","name":{"kind":"Name","value":"profileId"}},{"kind":"Field","name":{"kind":"Name","value":"codec"}},{"kind":"Field","name":{"kind":"Name","value":"displayInfo"}},{"kind":"Field","name":{"kind":"Name","value":"codecTag"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerSubtitleTrack"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"PlaybackSubtitleTrack"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"sourceTrackId"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"autoselect"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"languageBcp47"}},{"kind":"Field","name":{"kind":"Name","value":"renditions"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"variantId"}},{"kind":"Field","name":{"kind":"Name","value":"codec"}},{"kind":"Field","name":{"kind":"Name","value":"displayInfo"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerTimelinePreviewSheet"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"TimelinePreviewSheet"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"positionMs"}},{"kind":"Field","name":{"kind":"Name","value":"endMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetIntervalMs"}},{"kind":"Field","name":{"kind":"Name","value":"sheetGapSize"}},{"kind":"Field","name":{"kind":"Name","value":"asset"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"PlayerItemCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}}]}}]}}]} as unknown as DocumentNode<PlayerQuery, PlayerQueryVariables>;
export const SearchMediaDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SearchMedia"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"query"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"limit"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"kinds"}},"type":{"kind":"NonNullType","type":{"kind":"ListType","type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"NodeKind"}}}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"Variable","name":{"kind":"Name","value":"limit"}}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"searchTerm"},"value":{"kind":"Variable","name":{"kind":"Name","value":"query"}}},{"kind":"ObjectField","name":{"kind":"Name","value":"kinds"},"value":{"kind":"Variable","name":{"kind":"Name","value":"kinds"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"SearchNodeResult"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"SearchNodeResult"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"defaultFile"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"probe"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}}]}}]}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<SearchMediaQuery, SearchMediaQueryVariables>;
export const GetLibrariesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetLibraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<GetLibrariesQuery, GetLibrariesQueryVariables>;
export const CreateLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}},{"kind":"Argument","name":{"kind":"Name","value":"pinned"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<CreateLibraryMutation, CreateLibraryMutationVariables>;
export const UpdateLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"name"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"path"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Boolean"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}},{"kind":"Argument","name":{"kind":"Name","value":"name"},"value":{"kind":"Variable","name":{"kind":"Name","value":"name"}}},{"kind":"Argument","name":{"kind":"Name","value":"path"},"value":{"kind":"Variable","name":{"kind":"Name","value":"path"}}},{"kind":"Argument","name":{"kind":"Name","value":"pinned"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pinned"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"LibraryCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"LibraryCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Library"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"path"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastScannedAt"}}]}}]} as unknown as DocumentNode<UpdateLibraryMutation, UpdateLibraryMutationVariables>;
export const DeleteLibraryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteLibrary"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteLibrary"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"libraryId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"libraryId"}}}]}]}}]} as unknown as DocumentNode<DeleteLibraryMutation, DeleteLibraryMutationVariables>;
export const UsersManagementDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"UsersManagement"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"users"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UsersManagementQuery, UsersManagementQueryVariables>;
export const CreateUserInviteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"CreateUserInvite"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createUserInvite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"permissions"},"value":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<CreateUserInviteMutation, CreateUserInviteMutationVariables>;
export const UpdateUserDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateUser"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"updateUser"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}},{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"permissions"},"value":{"kind":"Variable","name":{"kind":"Name","value":"permissions"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<UpdateUserMutation, UpdateUserMutationVariables>;
export const ResetUserInviteDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"ResetUserInvite"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"resetUserInvite"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"UserCard"}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"UserCard"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"User"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"inviteCode"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastSeenAt"}}]}}]} as unknown as DocumentNode<ResetUserInviteMutation, ResetUserInviteMutationVariables>;
export const DeleteUserDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteUser"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"userId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"deleteUser"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"userId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"userId"}}}]}]}}]} as unknown as DocumentNode<DeleteUserMutation, DeleteUserMutationVariables>;
export const SidebarNavigationDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SidebarNavigation"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"libraries"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"pinned"}}]}}]}}]} as unknown as DocumentNode<SidebarNavigationQuery, SidebarNavigationQueryVariables>;
export const SidebarViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SidebarViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<SidebarViewerQuery, SidebarViewerQueryVariables>;
export const HomeRecentlyAddedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"HomeRecentlyAdded"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"24"}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"kinds"},"value":{"kind":"ListValue","values":[{"kind":"EnumValue","value":"MOVIE"},{"kind":"EnumValue","value":"SERIES"}]}},{"kind":"ObjectField","name":{"kind":"Name","value":"orderBy"},"value":{"kind":"EnumValue","value":"ADDED_AT"}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"currentPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<HomeRecentlyAddedQuery, HomeRecentlyAddedQueryVariables>;
export const HomeWatchedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"HomeWatched"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodeList"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"first"},"value":{"kind":"IntValue","value":"24"}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"kinds"},"value":{"kind":"ListValue","values":[{"kind":"EnumValue","value":"MOVIE"},{"kind":"EnumValue","value":"SERIES"},{"kind":"EnumValue","value":"EPISODE"}]}},{"kind":"ObjectField","name":{"kind":"Name","value":"watched"},"value":{"kind":"BooleanValue","value":true}},{"kind":"ObjectField","name":{"kind":"Name","value":"orderBy"},"value":{"kind":"EnumValue","value":"WATCH_PROGRESS_UPDATED_AT"}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"NodePoster"}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"NodePoster"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"displayDetail"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"firstAired"}},{"kind":"Field","name":{"kind":"Name","value":"lastAired"}}]}},{"kind":"Field","name":{"kind":"Name","value":"currentPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}}]}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"episodeCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}}]} as unknown as DocumentNode<HomeWatchedQuery, HomeWatchedQueryVariables>;
export const GetNodeByIdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetNodeById"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"node"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"nodeId"},"value":{"kind":"Variable","name":{"kind":"Name","value":"nodeId"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"kind"}},{"kind":"Field","name":{"kind":"Name","value":"unavailableAt"}},{"kind":"Field","name":{"kind":"Name","value":"unplayedCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonCount"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}},{"kind":"Field","name":{"kind":"Name","value":"parent"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}}]}},{"kind":"Field","name":{"kind":"Name","value":"root"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"FragmentSpread","name":{"kind":"Name","value":"GetPathForNode"}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"properties"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"tagline"}},{"kind":"Field","name":{"kind":"Name","value":"posterImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"thumbnailImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}}]}},{"kind":"Field","name":{"kind":"Name","value":"logoImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"aspectRatio"}}]}},{"kind":"Field","name":{"kind":"Name","value":"backdropImage"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"FragmentSpread","name":{"kind":"Name","value":"ImageAsset"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"aspectRatio"}}]}},{"kind":"Field","name":{"kind":"Name","value":"description"}},{"kind":"Field","name":{"kind":"Name","value":"contentRating"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"rating"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}},{"kind":"Field","name":{"kind":"Name","value":"currentPlayable"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"seasonNumber"}},{"kind":"Field","name":{"kind":"Name","value":"episodeNumber"}},{"kind":"Field","name":{"kind":"Name","value":"watchProgressHint"}}]}},{"kind":"Field","name":{"kind":"Name","value":"defaultFile"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"probe"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"runtimeMinutes"}},{"kind":"Field","name":{"kind":"Name","value":"width"}},{"kind":"Field","name":{"kind":"Name","value":"height"}},{"kind":"Field","name":{"kind":"Name","value":"videoCodec"}},{"kind":"Field","name":{"kind":"Name","value":"videoBitrate"}}]}}]}}]}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"GetPathForNode"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Node"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"libraryId"}}]}},{"kind":"FragmentDefinition","name":{"kind":"Name","value":"ImageAsset"},"typeCondition":{"kind":"NamedType","name":{"kind":"Name","value":"Asset"}},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"signedUrl"}},{"kind":"Field","name":{"kind":"Name","value":"thumbhash"}}]}}]} as unknown as DocumentNode<GetNodeByIdQuery, GetNodeByIdQueryVariables>;
export const PlaygroundViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"PlaygroundViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<PlaygroundViewerQuery, PlaygroundViewerQueryVariables>;
export const SettingsViewerDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"SettingsViewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"viewer"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"permissions"}}]}}]}}]} as unknown as DocumentNode<SettingsViewerQuery, SettingsViewerQueryVariables>;
export const SignupDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"Signup"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"username"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"password"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"inviteCode"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"String"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"signup"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"username"},"value":{"kind":"Variable","name":{"kind":"Name","value":"username"}}},{"kind":"Argument","name":{"kind":"Name","value":"password"},"value":{"kind":"Variable","name":{"kind":"Name","value":"password"}}},{"kind":"Argument","name":{"kind":"Name","value":"inviteCode"},"value":{"kind":"Variable","name":{"kind":"Name","value":"inviteCode"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}}]}}]} as unknown as DocumentNode<SignupMutation, SignupMutationVariables>;