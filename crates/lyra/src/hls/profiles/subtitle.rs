use super::{ProfileContext, StreamType, TranscodingProfile};
use easy_ffprobe::{Stream, StreamKinds};

pub struct WebVttSubtitleProfile;

impl TranscodingProfile for WebVttSubtitleProfile {
    fn stream_type(&self) -> StreamType {
        StreamType::Subtitle
    }

    fn name(&self) -> &str {
        "webvtt"
    }

    fn get_args(&self, context: &ProfileContext) -> Vec<String> {
        let seg_template = context
            .outdir
            .join("seg-%d.vtt")
            .to_string_lossy()
            .into_owned();
        let stream_map = format!("0:{}", context.stream_idx);

        #[rustfmt::skip]
        let args = vec![
            "-y".into(),
            "-ss".into(), context.start_time_offset.to_string(),
            "-i".into(), context.input_path.to_string_lossy().into(),
            "-copyts".into(),
            "-map".into(), stream_map,
            "-c:0".into(), "webvtt".into(),
            // "-start_at_zero".into(),
            // "-avoid_negative_ts".into(), "disabled".into(),
            "-f".into(), "segment".into(),
            "-segment_time".into(), context.segment_duration.to_string(),
            "-segment_list".into(), "pipe:1".into(),
            "-segment_list_type".into(), "m3u8".into(),
            "-segment_start_number".into(), context.segment_idx.to_string(),
            seg_template
        ];

        args
    }

    fn enable_for(&self, stream: &Stream) -> bool {
        match &stream.stream {
            StreamKinds::Subtitle(stream) => match stream.codec_name.as_str() {
                "webvtt" | "subrip" | "ass" | "ssa" => true,
                _ => false,
            },
            _ => false,
        }
    }
}
