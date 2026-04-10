use lyra_probe::{ProbeData, VideoKeyframes};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    KeyframeAligned,
    Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoProfileSelection {
    pub stream_index: u32,
    pub profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioProfileSelection {
    pub stream_index: u32,
    pub profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSpec {
    pub file_path: PathBuf,
    pub video: VideoProfileSelection,
    pub audio: Option<AudioProfileSelection>,
}

#[derive(Debug, Clone)]
pub struct SessionOptions {
    pub spec: SessionSpec,
    pub probe: ProbeData,
    pub keyframes: Option<VideoKeyframes>,
}
