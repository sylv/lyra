use crate::profiles::{ProfileContext, StreamType, TranscodingProfile};
use easy_ffprobe::{Stream, StreamKinds};

pub struct AacAudioProfile;

impl TranscodingProfile for AacAudioProfile {
    fn stream_type(&self) -> StreamType {
        StreamType::Audio
    }

    fn name(&self) -> &str {
        "aac"
    }

    fn get_args(&self, context: &ProfileContext) -> Vec<String> {
        let seg_template = context.outdir.join("%d.ts").to_string_lossy().into_owned();
        let playlist_path = context
            .outdir
            .join("playlist.m3u8")
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
            "-c:0".into(), "aac".into(),
            "-ac".into(), "2".into(),
            "-ab".into(), "128k".into(),
            "-start_at_zero".into(),
            "-avoid_negative_ts".into(), "make_non_negative".into(),
            "-f".into(), "hls".into(),
            "-start_number".into(), context.segment_idx.to_string(),
            "-hls_flags".into(), "temp_file".into(),
            "-hls_time".into(), context.segment_duration.to_string(),
            "-hls_segment_filename".into(), seg_template,
            playlist_path
        ];

        args
    }

    fn enable_for(&self, stream: &Stream) -> bool {
        match stream.stream {
            StreamKinds::Audio(_) => true,
            _ => false,
        }
    }
}
