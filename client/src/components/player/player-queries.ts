import { graphql } from "../../@generated/gql";

export const ItemPlaybackQuery = graphql(`
  query ItemPlayback($itemId: String!) {
    node(nodeId: $itemId) {
      id
      libraryId
      kind
      properties {
        displayName
        seasonNumber
        episodeNumber
        firstAired
        lastAired
      }
      root {
        id
        libraryId
        properties {
          displayName
        }
      }
      watchProgress {
        id
        progressPercent
        completed
        updatedAt
      }
      defaultFile {
        id
        probe {
          runtimeMinutes
        }
        playbackOptions {
          videoRenditions {
            renditionId
            displayName
            displayInfo
            codecTag
            onDemand
          }
          audioTracks {
            streamIndex
            displayName
            language
            recommended
            renditions {
              renditionId
              codecName
              bitrate
              channels
              sampleRate
              codecTag
              onDemand
            }
          }
          subtitleTracks {
            subtitleId
            streamIndex
            displayName
            language
            recommended
            renditions {
              renditionId
              codecName
              onDemand
            }
          }
        }
        subtitleTracks {
          id
          streamIndex
          kind
          source
          label
          language
          dispositions
          derivedFromSubtitleId
          asset {
            id
            signedUrl
          }
        }
        recommendedSubtitleTrackId
        segments {
          kind
          startMs
          endMs
        }
        timelinePreview {
          ...PlayerTimelinePreviewSheet
        }
      }
      previousPlayable {
        id
        properties {
          displayName
          description
          thumbnailImage {
            ...ImageAsset
          }
          seasonNumber
          episodeNumber
        }
      }
      nextPlayable {
        id
        properties {
          displayName
          description
          thumbnailImage {
            ...ImageAsset
          }
          seasonNumber
          episodeNumber
        }
      }
      ...GetPathForNode
    }
  }
`);

export const MintPlaybackUrl = graphql(`
  mutation MintPlaybackUrl($input: PlaybackUrlInput!) {
    mintPlaybackUrl(input: $input) {
      url
      packagerId
    }
  }
`);

export const UpdateWatchState = graphql(`
  mutation UpdateWatchState($fileId: String!, $progressPercent: Float!) {
    updateWatchProgress(fileId: $fileId, progressPercent: $progressPercent) {
      progressPercent
      updatedAt
    }
  }
`);

export const SetPreferredAudio = graphql(`
  mutation SetPreferredAudio($language: String, $disposition: TrackDispositionPreference) {
    setPreferredAudio(language: $language, disposition: $disposition) {
      id
      preferredAudioLanguage
      preferredAudioDisposition
    }
  }
`);

export const SetPreferredSubtitle = graphql(`
  mutation SetPreferredSubtitle($language: String, $disposition: TrackDispositionPreference) {
    setPreferredSubtitle(language: $language, disposition: $disposition) {
      id
      preferredSubtitleLanguage
      preferredSubtitleDisposition
    }
  }
`);

export const WatchSessionSummary = graphql(`
  fragment WatchSessionSummary on WatchSession {
    id
    nodeId
    fileId
    mode
    intent
    effectiveState
    currentPositionMs
    basePositionMs
    baseTimeMs
    revision
    players {
      id
      userId
      user {
        id
        username
      }
      isBuffering
      isInactive
      canRemove
    }
  }
`);

export const WatchSessionBeaconFragment = graphql(`
  fragment WatchSessionBeaconFragment on WatchSessionBeacon {
    sessionId
    nodeId
    fileId
    mode
    intent
    effectiveState
    basePositionMs
    baseTimeMs
    revision
    players {
      id
      userId
      user {
        id
        username
      }
      isBuffering
      isInactive
      canRemove
    }
  }
`);

export const WatchSessionViewer = graphql(`
  query WatchSessionViewer {
    viewer {
      id
      permissions
    }
  }
`);

export const GetWatchSession = graphql(`
  query GetWatchSession($sessionId: String!) {
    watchSession(sessionId: $sessionId) {
      ...WatchSessionSummary
    }
  }
`);

export const LeaveWatchSession = graphql(`
  mutation LeaveWatchSession($sessionId: String!, $playerId: String!) {
    leaveWatchSession(sessionId: $sessionId, playerId: $playerId)
  }
`);

export const WatchSessionHeartbeat = graphql(`
  mutation WatchSessionHeartbeat($input: WatchSessionHeartbeatInput!) {
    watchSessionHeartbeat(input: $input) {
      ...WatchSessionBeaconFragment
    }
  }
`);

export const WatchSessionAction = graphql(`
  mutation WatchSessionAction($input: WatchSessionActionInput!) {
    watchSessionAction(input: $input) {
      ...WatchSessionBeaconFragment
    }
  }
`);

export const WatchSessionBeacons = graphql(`
  subscription WatchSessionBeacons($sessionId: String!, $playerId: String!) {
    watchSessionBeacons(sessionId: $sessionId, playerId: $playerId) {
      ...WatchSessionBeaconFragment
    }
  }
`);
