use crate::{
    config::TARGET_SEGMENT_SECONDS,
    model::StreamType,
    profiles::{Profile, ProfileContext, ProfileType, SegmentLayout},
};
use std::ffi::OsString;

#[derive(Debug)]
pub struct AudioAacProfile;

impl Profile for AudioAacProfile {
    fn display_name(&self) -> &'static str {
        "Convert to AAC"
    }

    fn id_name(&self) -> &'static str {
        "audio_aac"
    }

    fn profile_type(&self) -> ProfileType {
        ProfileType::Transcode
    }

    fn segment_layout(&self) -> SegmentLayout {
        SegmentLayout::Fixed
    }

    fn stream_type(&self) -> StreamType {
        StreamType::Audio
    }

    fn supports_stream(&self, ctx: &ProfileContext) -> bool {
        ctx.stream.stream_type == StreamType::Audio
    }

    fn build_args(
        &self,
        ctx: &ProfileContext,
        start_segment: i64,
        start_seconds: f64,
        _hls_cuts: &str,
    ) -> Vec<OsString> {
        let mut a: Vec<OsString> = Vec::new();

        if start_segment > 0 {
            // unlike video/copy, we can seek to the exact start position using exact seeks.
            ffarg!(a, "-ss", format!("{start_seconds:.6}"));
        }

        ffarg!(a, "-i", ctx.input.clone().into_os_string());

        // take just the stream we want
        ffarg!(a, "-map", format!("0:{}", ctx.stream.stream_index));

        // we target aac with broad compatibility for simplicity
        ffarg!(a, "-codec:a", "aac");
        ffarg!(a, "-profile:a", "aac_low");
        ffarg!(a, "-ac", "2");
        ffarg!(a, "-b:a", "160k");

        // copy original timestamps so our segment boundaries (from keyframes) align.
        // because we're transcoding this isnt necessary, but if we don't our timestamps
        // may not align to the video timestamps, which hls.js does not like at all
        ffarg!(a, "-copyts");
        ffarg!(a, "-avoid_negative_ts", "make_non_negative");

        // hls stuff
        ffarg!(a, "-f", "hls");
        ffarg!(a, "-start_number", start_segment.to_string());
        ffarg!(a, "-hls_time", TARGET_SEGMENT_SECONDS.to_string());
        // ffarg!(a, "-hls_cuts", hls_cuts);
        ffarg!(a, "-hls_flags", "temp_file");
        ffarg!(a, "-hls_segment_filename", "%d.m4s");
        ffarg!(a, "-hls_segment_options", "movflags=+frag_discont");
        ffarg!(a, "-hls_segment_type", "fmp4");

        ffarg!(a, "pipe:1");

        a
    }
}
