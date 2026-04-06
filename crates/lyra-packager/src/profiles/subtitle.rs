use crate::{
    model::StreamType,
    profiles::{PlaylistKind, Profile, ProfileContext, ProfileType, SegmentLayout},
};
use lyra_probe::Codec;
use std::ffi::OsString;

#[derive(Debug)]
pub struct SubtitleWebVttProfile;

impl Profile for SubtitleWebVttProfile {
    fn display_name(&self) -> &'static str {
        "Convert to WebVTT"
    }

    fn id_name(&self) -> &'static str {
        "subtitle_webvtt"
    }

    fn profile_type(&self) -> ProfileType {
        ProfileType::Transcode
    }

    fn segment_layout(&self) -> SegmentLayout {
        SegmentLayout::Single
    }

    fn playlist_kind(&self) -> PlaylistKind {
        PlaylistKind::WebVtt
    }

    fn stream_type(&self) -> StreamType {
        StreamType::Subtitle
    }

    fn supports_stream(&self, ctx: &ProfileContext) -> bool {
        if ctx.stream.stream_type != StreamType::Subtitle {
            return false;
        }

        // only advertise text subtitles here. image-based codecs like pgs/dvdsub
        // won't reliably transcode to webvtt in this simple one-shot path.
        matches!(
            ctx.stream.codec,
            Codec::SubtitleAss
                | Codec::SubtitleMovText
                | Codec::SubtitleSubRip
                | Codec::SubtitleText
                | Codec::SubtitleTtml
                | Codec::SubtitleWebVtt
        )
    }

    fn build_args(
        &self,
        ctx: &ProfileContext,
        _start_segment: i64,
        _start_seconds: f64,
        _hls_cuts: &str,
    ) -> Vec<OsString> {
        let mut a: Vec<OsString> = Vec::new();

        ffarg!(a, "-i", ctx.input.clone().into_os_string());
        ffarg!(a, "-map", format!("0:{}", ctx.stream.stream_index));

        // keep subtitle output as a single webvtt file so the rendition stays simple and
        // we don't need timestamp remapping across many text segments.
        ffarg!(a, "-codec:s", "webvtt");
        ffarg!(a, "-f", "webvtt");
        ffarg!(a, "-y");
        ffarg!(a, "0.vtt");

        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StreamDescriptor;
    use std::{path::PathBuf, sync::Arc};

    fn ctx(codec: Codec) -> ProfileContext {
        ProfileContext {
            input: PathBuf::new(),
            stream: StreamDescriptor {
                stream_id: 0,
                stream_index: 0,
                stream_type: StreamType::Subtitle,
                codec,
                bit_rate: None,
                frame_rate: None,
                width: None,
                height: None,
                channels: None,
                language: None,
                is_primary_video: false,
                is_forced: false,
                is_sdh: false,
                is_commentary: false,
                display_name: "".to_string(),
            },
            stream_info: None,
            keyframes: Some(Arc::new(Vec::new())),
        }
    }

    #[test]
    fn supports_text_subtitles_only() {
        let profile = SubtitleWebVttProfile;

        assert!(profile.supports_stream(&ctx(Codec::SubtitleSubRip)));
        assert!(profile.supports_stream(&ctx(Codec::SubtitleAss)));
        assert!(!profile.supports_stream(&ctx(Codec::SubtitlePgs)));
        assert!(!profile.supports_stream(&ctx(Codec::SubtitleVobSub)));
    }
}
