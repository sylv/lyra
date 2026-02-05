use crate::{
    config::TARGET_SEGMENT_SECONDS,
    model::StreamType,
    profiles::{Profile, ProfileContext, ProfileType},
};
use std::ffi::OsString;

#[derive(Debug)]
pub struct VideoCopyProfile;

impl Profile for VideoCopyProfile {
    fn display_name(&self) -> &'static str {
        "Original"
    }

    fn id_name(&self) -> &'static str {
        "video_copy"
    }

    fn profile_type(&self) -> ProfileType {
        ProfileType::Copy
    }

    fn stream_type(&self) -> StreamType {
        StreamType::Video
    }

    fn supports_stream(&self, ctx: &ProfileContext) -> bool {
        if ctx.stream.stream_type != StreamType::Video {
            return false;
        }
        if !ctx.stream.is_primary_video {
            return false;
        }
        if ctx.keyframes.is_none() || ctx.stream_info.is_none() {
            return false;
        }
        matches!(ctx.stream.codec_name.as_str(), "h264" | "hevc" | "av1")
    }

    fn build_args(
        &self,
        ctx: &ProfileContext,
        start_segment: i64,
        start_seconds: f64,
    ) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();

        if start_segment > 0 {
            // We bump by 0.5s to bias towards the keyframe that starts the target segment.
            // The stream copy path relies on keyframe-aligned seeks; this reduces off-by-one
            // segment selection when timestamps are slightly earlier than the desired boundary.
            let seek_seconds = start_seconds + 0.5;
            args.push("-ss".into());
            args.push(format!("{seek_seconds:.6}").into());
            args.push("-noaccurate_seek".into());
        }

        // Input after -ss means "fast seek" (keyframe seek) for copy-mode segments.
        args.extend(["-i".into(), ctx.input.clone().into_os_string()]);

        {
            #[rustfmt::skip]
            args.extend([
                // copy instead of transcode
                "-codec:v", "copy",
                // copy original timestamps so our segment boundaries (from keyframes) align.
                // avoid_negative_ts=disabled keeps ffmpeg from shifting timestamps to start at 0,
                // which would desync from the keyframe-derived playlist positions.
                "-copyts",
                "-avoid_negative_ts", "disabled",
                // "ts" is common but doesn't support AV1/HEVC
                "-hls_segment_type", "fmp4",
            ].into_iter().map(Into::into));
        }

        // take just the stream we want
        args.push("-map".into());
        args.push(format!("0:{}", ctx.stream.stream_index).into());

        args.extend([
            "-hls_time".into(),
            TARGET_SEGMENT_SECONDS.to_string().into(),
        ]);
        args.extend(["-start_number".into(), start_segment.to_string().into()]);

        {
            #[rustfmt::skip]
            args.extend([
                "-f", "hls",
                "-hls_segment_filename", "%d.m4s",
                "-hls_fmp4_init_filename", "init.mp4",
                "-hls_segment_options", "movflags=+frag_discont",
                "-hls_playlist_type", "vod",
                "-hls_list_size", "0",
                "-y",
                "pipe:1",
            ].into_iter().map(Into::into));
        }

        args
    }
}
