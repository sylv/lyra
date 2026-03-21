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
    pub bit_rate: Option<u64>,
    pub frame_rate: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub channels: Option<u32>,
    pub language: Option<String>,
    pub is_primary_video: bool,
    pub is_forced: bool,
    pub is_sdh: bool,
    pub is_commentary: bool,
    pub display_name: String,
}

#[derive(Clone, Debug)]
pub struct StreamInfo {
    pub time_base_num: i64,
    pub time_base_den: i64,
    pub duration_seconds: f64,
}
