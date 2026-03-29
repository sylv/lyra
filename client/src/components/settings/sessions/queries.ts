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
			user {
				id
				username
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
				runtimeMinutes
				releasedAt
				endedAt
				posterImage {
					...ImageAsset
				}
				thumbnailImage {
					...ImageAsset
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
