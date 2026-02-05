use crate::{
    config::TARGET_SEGMENT_SECONDS,
    model::StreamType,
    profiles::{Profile, ProfileContext, ProfileType},
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
    ) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();

        if start_segment > 0 {
            // Audio seeks are timestamp-based; keep the exact requested start.
            args.push("-ss".into());
            args.push(format!("{start_seconds:.6}").into());
        }

        args.extend(["-i".into(), ctx.input.clone().into_os_string()]);

        {
            #[rustfmt::skip]
            args.extend([
                // we target aac with broad compatibility for simplicity
                "-codec:a", "aac",
                "-profile:a", "aac_low",
                "-ac", "2",
                "-b:a", "160k",
                // copy original timestamps so our segment boundaries (from keyframes) align.
                // because we're transcoding this isnt necessary, but if we don't our timestamps
                // may not align to the video timestamps, which hls.js does not like at all
                "-copyts",
                "-avoid_negative_ts", "disabled",
                // fmp4 for broad compatibility
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
