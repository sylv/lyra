import type { ItemPlaybackQuery } from "../../../@generated/gql/graphql";
import { videoState } from "../video-state";
import { setPlayerLoading } from "../player-state";

type ServerTracks = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["tracks"]>;
type Recommendations = NonNullable<NonNullable<NonNullable<ItemPlaybackQuery["node"]>["file"]>["recommendedTracks"]>;

export interface ResumeConfig {
	watchProgressPercent: number | null | undefined;
	runtimeDurationSeconds: number | null;
	shouldPromptResume: boolean;
	videoRef: React.RefObject<HTMLVideoElement | null>;
}

export interface PlaybackEngine {
	setAudioTrack(id: number): void;
	setSubtitleTrack(id: number): void;
	setSubtitleDisplay(enabled: boolean): void;
	destroy(): void;
}

export const createPlaybackEngine = async (
	video: HTMLVideoElement,
	hlsUrl: string,
	serverTracks: ServerTracks,
	recommendations: Recommendations,
	resumeConfig: ResumeConfig,
): Promise<PlaybackEngine | null> => {
	// hls.js is more reliable for our setup. specifically, i believe chrome doesnt like us not reporting
	// the "codecs" tag in the playlist, so it breaks.
	// todo: it might make more sense to try use native hls, falling back to hls.js on error, then erroring if neither work,
	// or reporting codecs properly
	const { default: Hls } = await import("hls.js");
	if (Hls.isSupported()) {
		const { createHlsJsEngine } = await import("./hlsjs-engine");
		return createHlsJsEngine(Hls, video, hlsUrl, serverTracks, recommendations, resumeConfig);
	}

	if (video.canPlayType("application/vnd.apple.mpegurl")) {
		const { createNativeEngine } = await import("./native-engine");
		return createNativeEngine(video, hlsUrl, serverTracks, recommendations, resumeConfig);
	}

	videoState.setState({ errorMessage: "Sorry, your browser does not support this video." });
	setPlayerLoading(false);
	return null;
};
