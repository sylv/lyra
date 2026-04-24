import { graphql } from "../../../@generated/gql";

export const SessionCardFragment = graphql(`
  fragment SessionCard on WatchSession {
    id
    updatedAt
    currentPositionMs
    effectiveState
    players {
      id
      userId
      displayUsername
      user {
        id
        createdAt
      }
    }
    node {
      id
      libraryId
      properties {
        displayName
        seasonNumber
        episodeNumber
        firstAired
        lastAired
        posterImage {
          ...ImageAsset
        }
        thumbnailImage {
          ...ImageAsset
        }
      }
      defaultFile {
        probe {
          runtimeMinutes
        }
      }
      root {
        id
        properties {
          displayName
          posterImage {
            ...ImageAsset
          }
        }
      }
    }
    file {
      id
      timelinePreview {
        ...PlayerTimelinePreviewSheet
      }
    }
  }
`);

export const SessionsQuery = graphql(`
  query SettingsSessions {
    watchSessions {
      id
      ...SessionCard
    }
  }
`);
