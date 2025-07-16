use easy_ffprobe::Stream;
use std::path::PathBuf;

pub mod audio;
pub mod video;

pub enum StreamType {
    Video,
    Audio,
}

impl StreamType {
    pub fn as_str(&self) -> &str {
        match self {
            StreamType::Video => "video",
            StreamType::Audio => "audio",
        }
    }

    pub fn from_str(s: &str) -> Option<StreamType> {
        match s {
            "video" => Some(StreamType::Video),
            "audio" => Some(StreamType::Audio),
            _ => None,
        }
    }
}

pub struct ProfileContext {
    pub input_path: PathBuf,
    pub stream: Stream,
    pub outdir: PathBuf,
    pub segment_idx: usize,
    pub segment_duration: f64,
    pub start_time_offset: f64,
    pub stream_idx: usize,
}

pub trait TranscodingProfile {
    fn stream_type(&self) -> StreamType;
    fn name(&self) -> &str;
    fn get_args(&self, context: &ProfileContext) -> Vec<String>;
    fn enable_for(&self, stream: &Stream) -> bool;
}
