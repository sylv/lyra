use super::{ProfileContext, StreamType, TranscodingProfile};
use easy_ffprobe::{Stream, StreamKinds};

pub struct CopyVideoProfile;

impl TranscodingProfile for CopyVideoProfile {
    fn stream_type(&self) -> StreamType {
        StreamType::Video
    }

    fn name(&self) -> &str {
        "copy"
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
            "-c:0".into(), "copy".into(),
            "-start_at_zero".into(),
            "-avoid_negative_ts".into(), "disabled".into(),
            "-f".into(), "hls".into(),
            "-start_number".into(), context.segment_idx.to_string(),
            "-hls_flags".into(), "split_by_time+temp_file".into(),
            "-hls_time".into(), context.segment_duration.to_string(),
            "-hls_segment_filename".into(), seg_template,
            playlist_path
        ];

        args
    }

    fn enable_for(&self, stream: &Stream) -> bool {
        match &stream.stream {
            StreamKinds::Video(stream) => match stream.codec_name.as_str() {
                "h264" | "hevc" => true,
                _ => false,
            },
            _ => false,
        }
    }
}

pub struct H264VideoProfile;

impl TranscodingProfile for H264VideoProfile {
    fn stream_type(&self) -> StreamType {
        StreamType::Video
    }

    fn name(&self) -> &str {
        "h264"
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
            "-c:0".into(), "libx264".into(),
            "-preset".into(), "veryfast".into(),
            "-start_at_zero".into(),
            "-avoid_negative_ts".into(), "make_non_negative".into(),
            "-f".into(), "hls".into(),
            "-start_number".into(), context.segment_idx.to_string(),
            "-hls_flags".into(), "temp_file".into(),
            "-hls_time".into(), context.segment_duration.to_string(),
            "-hls_segment_filename".into(), seg_template,
            "-force_key_frames".into(), format!("expr:gte(t,n_forced*{})", context.segment_duration),
            playlist_path
        ];

        args
    }

    fn enable_for(&self, stream: &Stream) -> bool {
        match &stream.stream {
            StreamKinds::Video(_) => true,
            _ => false,
        }
    }
}
