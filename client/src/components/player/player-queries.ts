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
				runtimeMinutes
			}
			root {
				libraryId
				properties {
					displayName
				}
			}
			watchProgress {
				progressPercent
				completed
				updatedAt
			}
			file {
				id
				tracks {
					trackIndex
					manifestIndex
					trackType
					displayName
					language
					disposition
					isForced
				}
				recommendedTracks {
					manifestIndex
					trackType
					enabled
				}
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
					thumbnailImage { ...ImageAsset }
					seasonNumber
					episodeNumber
				}
			}
			nextPlayable {
				id
				properties {
					displayName
					description
					thumbnailImage { ...ImageAsset }
					seasonNumber
					episodeNumber
				}
			}
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
