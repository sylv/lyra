import type { ResultOf } from "@graphql-typed-document-node/core";
import { persist } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";
import { create } from "zustand/react";
import { graphql } from "../../../@generated/gql";
import type {
  PlayerAudioTrackFragment,
  PlayerSubtitleTrackFragment,
  PlayerVideoTrackFragment,
} from "../../../@generated/gql/graphql";

export enum PlayerState {
  Hidden = "hidden",
  Init = "init",
  Resuming = "resuming",
  Mounted = "mounted",
  Error = "error",
}

export interface PlayerStore {
  status: PlayerStatus;
  targetNodeId: string | null;
  videoRef: React.RefObject<HTMLVideoElement | null>;
  currentTime: number;
  durationSeconds: number;
  bufferedRanges: Array<{ start: number; end: number }>;
  aspectRatio: number;
  developerMode: boolean;
  isFullscreen: boolean;
  buffering: boolean;
  paused: boolean;
  ended: boolean;
  volume: number;
  muted: boolean;
  playbackRate: number;
  selectedVideoRenditionPairId: string | null;
  selectedAudioTrackId: string | null;
  selectedSubtitleTrackId: string | null;
  activeSubtitleTrackId: string | null;
  activeSubtitleRenditionId: string | null;
  pendingSubtitleTrackId: string | null;
  videoRenditionOptions: PlayerVideoRenditionOption[];
  audioTrackOptions: PlayerAudioTrackOption[];
}

export interface PlayerVideoRenditionOption {
  track: PlayerVideoTrackFragment;
  rendition: PlayerVideoTrackFragment["renditions"][number];
  compatibility: CanPlayTypeResult;
}

export interface PlayerAudioTrackOption {
  track: PlayerAudioTrackFragment;
  supportedRenditions: Array<{
    rendition: PlayerAudioTrackFragment["renditions"][number];
    compatibility: CanPlayTypeResult;
  }>;
}

export const usePlayerStore = create<PlayerStore>()(
  persist(
    immer(() => ({
      status: { state: PlayerState.Hidden },
      targetNodeId: null,
      currentTime: 0,
      durationSeconds: 0,
      bufferedRanges: [],
      aspectRatio: 16 / 9,
      developerMode: import.meta.env.DEV,
      videoRef: { current: null },
      isFullscreen: false,
      buffering: true,
      paused: true,
      ended: false,
      volume: 1,
      muted: false,
      playbackRate: 1,
      selectedVideoRenditionPairId: null,
      selectedAudioTrackId: null,
      selectedSubtitleTrackId: null,
      activeSubtitleTrackId: null,
      activeSubtitleRenditionId: null,
      pendingSubtitleTrackId: null,
      videoRenditionOptions: [],
      audioTrackOptions: [],
    })),
    {
      name: "lyra.player",
      partialize: (state) => ({
        volume: state.volume,
        muted: state.muted,
      }),
    },
  ),
);

export type PlayerStatus =
  | { state: PlayerState.Hidden }
  | { state: PlayerState.Init }
  | {
      state: PlayerState.Resuming;
      fromTimeMs: number;
      data: ResultOf<typeof PlayerQuery>;
      videoTrack: PlayerVideoTrackFragment;
      videoTracks: Array<PlayerVideoTrackFragment>;
      audioTrack: PlayerAudioTrackFragment | null;
      audioTracks: Array<PlayerAudioTrackFragment>;
      subtitleTracks: Array<PlayerSubtitleTrackFragment>;
    }
  | {
      state: PlayerState.Mounted;
      data: ResultOf<typeof PlayerQuery>;
      videoTrack: PlayerVideoTrackFragment;
      videoTracks: Array<PlayerVideoTrackFragment>;
      audioTrack: PlayerAudioTrackFragment | null;
      audioTracks: Array<PlayerAudioTrackFragment>;
      subtitleTracks: Array<PlayerSubtitleTrackFragment>;
    }
  | { state: PlayerState.Error; errorMessage: string };

export const setPlayerStatus = (status: PlayerStatus) => {
  usePlayerStore.setState({ status });
};

export const setPlayerError = (errorMessage: string) => {
  setPlayerStatus({ state: PlayerState.Error, errorMessage });
};

export const resetPlayer = () => {
  const { volume, muted, videoRef } = usePlayerStore.getState();
  videoRef.current?.pause();
  usePlayerStore.setState({ ...usePlayerStore.getInitialState(), volume, muted }, true);
};

export const playNode = (nodeId: string, autoFullscreen: boolean) => {
  usePlayerStore.setState((state) => {
    state.targetNodeId = nodeId;
    if (autoFullscreen && state.isFullscreen === false) {
      state.isFullscreen = true;
    }
  });
};

export const closePlayer = () => {
  if (document.fullscreenElement) {
    document.exitFullscreen().catch(() => undefined);
  }
  resetPlayer();
};

export const setPlayerVolume = (volume: number) => {
  const safeVolume = Math.max(0, Math.min(1, volume));
  usePlayerStore.setState((state) => {
    state.volume = safeVolume;
    if (safeVolume > 0) state.muted = false;
  });
};

export const togglePlayerMute = () => {
  usePlayerStore.setState((state) => {
    state.muted = !state.muted;
  });
};

export const setPlayerMuted = (muted: boolean) => {
  usePlayerStore.setState({ muted });
};

export const PlayerQuery = graphql(`
  query Player($nodeId: String!) {
    node(nodeId: $nodeId) {
      id
      ...GetPathForNode
      properties {
        displayName
        seasonNumber
        episodeNumber
      }
      root {
        properties {
          displayName
        }
      }
      defaultFile {
        id
        height
        width
        resumeHint {
          startMs
          updatedAt
        }
        probe {
          durationSeconds
          width
          height
        }
        playback {
          hlsUrlTemplate
          video {
            ...PlayerVideoTrack
          }
          audio {
            ...PlayerAudioTrack
          }
          subtitles {
            ...PlayerSubtitleTrack
          }
        }
        timelinePreview {
          ...PlayerTimelinePreviewSheet
        }
        segments {
          kind
          startMs
          endMs
        }
      }
      previousPlayable {
        id
        ...PlayerItemCard
      }
      nextPlayable {
        id
        ...PlayerItemCard
      }
    }
  }
`);

export const PlayerAudioTrack = graphql(`
  fragment PlayerAudioTrack on PlaybackAudioTrack {
    sourceTrackId
    displayName
    autoselect
    languageBcp47
    renditions {
      pairId
      profileId
      codec
      displayInfo
      codecTag
    }
  }
`);

export const PlayerVideoTrack = graphql(`
  fragment PlayerVideoTrack on PlaybackVideoTrack {
    sourceTrackId
    displayName
    autoselect
    renditions {
      pairId
      profileId
      codec
      displayInfo
      codecTag
    }
  }
`);

export const PlayerSubtitleTrack = graphql(`
  fragment PlayerSubtitleTrack on PlaybackSubtitleTrack {
    sourceTrackId
    displayName
    autoselect
    kind
    languageBcp47
    renditions {
      variantId
      codec
      displayInfo
      signedUrl
    }
  }
`);

export const PlayerTimelinePreviewSheet = graphql(`
  fragment PlayerTimelinePreviewSheet on TimelinePreviewSheet {
    positionMs
    endMs
    sheetIntervalMs
    sheetGapSize
    asset {
      id
      signedUrl
      width
      height
    }
  }
`);

export const PlayerItemCard = graphql(`
  fragment PlayerItemCard on Node {
    id
    ...GetPathForNode
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
`);
