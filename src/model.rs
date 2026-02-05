#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamType {
    Video,
    Audio,
    Subtitle,
}

#[derive(Clone, Debug)]
pub struct StreamDescriptor {
    pub stream_id: u32,
    pub stream_index: u32,
    pub stream_type: StreamType,
    pub codec_name: String,
    pub language: Option<String>,
    pub is_primary_video: bool,
}

#[derive(Clone, Debug)]
pub struct StreamInfo {
    pub time_base_num: i64,
    pub time_base_den: i64,
    pub duration_seconds: f64,
}
