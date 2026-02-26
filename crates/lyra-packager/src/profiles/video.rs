use crate::{
    config::TARGET_SEGMENT_SECONDS,
    model::StreamType,
    profiles::{Profile, ProfileContext, ProfileType, SegmentLayout},
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

    fn segment_layout(&self) -> SegmentLayout {
        SegmentLayout::Keyframe
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
        hls_cuts: &str,
    ) -> Vec<OsString> {
        let mut a: Vec<OsString> = Vec::new();

        if start_segment > 0 {
            // We bump by 0.05s to bias towards the keyframe that starts the target segment.
            // The stream copy path relies on keyframe-aligned seeks; this reduces off-by-one
            // segment selection when timestamps are slightly earlier than the desired boundary.
            let seek_seconds = start_seconds + 0.05;
            ffarg!(a, "-ss", format!("{seek_seconds:.6}"));
            ffarg!(a, "-noaccurate_seek");
        }

        // Input after -ss means "fast seek" (keyframe seek) for copy-mode segments.
        ffarg!(a, "-i", ctx.input.clone().into_os_string());

        // take just the stream we want
        ffarg!(a, "-map", format!("0:{}", ctx.stream.stream_index));

        // copy instead of transcode
        ffarg!(a, "-codec:v", "copy");

        // copy original timestamps so our segment boundaries (from keyframes) align.
        // avoid_negative_ts=disabled keeps ffmpeg from shifting timestamps to start at 0,
        // which would desync from the keyframe-derived playlist positions.
        ffarg!(a, "-copyts");
        ffarg!(a, "-avoid_negative_ts", "make_non_negative");

        // hls stuff
        ffarg!(a, "-f", "hls");
        ffarg!(a, "-hls_time", TARGET_SEGMENT_SECONDS.to_string());
        ffarg!(a, "-hls_cuts", hls_cuts);
        ffarg!(a, "-start_number", start_segment.to_string());
        ffarg!(a, "-hls_flags", "temp_file");
        ffarg!(a, "-hls_segment_type", "fmp4");
        ffarg!(a, "-hls_segment_filename", "%d.m4s");
        ffarg!(a, "-hls_segment_options", "movflags=+frag_discont");

        ffarg!(a, "pipe:1");

        a
    }
}

#[derive(Debug)]
pub struct VideoH264Profile;

impl Profile for VideoH264Profile {
    fn display_name(&self) -> &'static str {
        "Convert"
    }

    fn id_name(&self) -> &'static str {
        "video_h264"
    }

    fn profile_type(&self) -> ProfileType {
        ProfileType::Transcode
    }

    fn segment_layout(&self) -> SegmentLayout {
        SegmentLayout::Fixed
    }

    fn stream_type(&self) -> StreamType {
        StreamType::Video
    }

    fn supports_stream(&self, ctx: &ProfileContext) -> bool {
        if !ctx.stream.is_primary_video {
            return false;
        }

        ctx.stream.stream_type == StreamType::Video
    }

    fn build_args(
        &self,
        ctx: &ProfileContext,
        start_segment: i64,
        start_seconds: f64,
        hls_cuts: &str,
    ) -> Vec<OsString> {
        let mut a: Vec<OsString> = Vec::new();

        if start_segment > 0 {
            // unlike copy, we can seek to the exact start position using exact seeks.
            ffarg!(a, "-ss", format!("{start_seconds:.6}"));
        }

        // Keep -ss before the input so startup seeks avoid decoding from the beginning.
        ffarg!(a, "-i", ctx.input.clone().into_os_string());

        // copy instead of transcode
        ffarg!(a, "-codec:v", "libx264");
        ffarg!(a, "-preset", "veryfast");
        ffarg!(
            a,
            "-force_key_frames",
            format!("expr:gte(t,n_forced*{})", TARGET_SEGMENT_SECONDS)
        );

        // Preserve source timestamps so segment timing remains stable across ffmpeg restarts.
        ffarg!(a, "-copyts");
        ffarg!(a, "-avoid_negative_ts", "make_non_negative");

        // take just the stream we want
        ffarg!(a, "-map", format!("0:{}", ctx.stream.stream_index));

        // hls stuff
        ffarg!(a, "-f", "hls");
        ffarg!(a, "-hls_time", TARGET_SEGMENT_SECONDS.to_string());
        ffarg!(a, "-hls_cuts", hls_cuts);
        ffarg!(a, "-start_number", start_segment.to_string());
        ffarg!(a, "-hls_flags", "temp_file");
        ffarg!(a, "-hls_segment_type", "fmp4");
        ffarg!(a, "-hls_segment_filename", "%d.m4s");
        ffarg!(a, "-hls_fmp4_init_filename", "init.mp4");
        ffarg!(a, "-hls_segment_options", "movflags=+frag_discont");
        ffarg!(a, "-hls_playlist_type", "vod");
        ffarg!(a, "-hls_list_size", "0");

        ffarg!(a, "-y");
        ffarg!(a, "pipe:1");

        a
    }
}
