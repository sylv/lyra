use crate::model::{StreamDescriptor, StreamInfo, StreamType};
use std::{ffi::OsString, path::PathBuf, sync::Arc};

pub mod audio;
pub use audio::AudioAacProfile;

pub mod video;
pub use video::VideoCopyProfile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileType {
    Copy,
    Transcode,
}

pub struct ProfileContext {
    pub input: PathBuf,
    pub stream: StreamDescriptor,
    pub stream_info: Option<StreamInfo>,
    pub keyframes: Option<Arc<Vec<f64>>>,
}

pub trait Profile: Send + Sync {
    fn display_name(&self) -> &'static str;
    fn id_name(&self) -> &'static str;
    fn profile_type(&self) -> ProfileType;
    fn stream_type(&self) -> StreamType;
    fn supports_stream(&self, ctx: &ProfileContext) -> bool;
    fn build_args(
        &self,
        ctx: &ProfileContext,
        start_segment: i64,
        start_seconds: f64,
    ) -> Vec<OsString>;
}
