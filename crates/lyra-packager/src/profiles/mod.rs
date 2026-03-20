use crate::model::{StreamDescriptor, StreamInfo, StreamType};
use std::{ffi::OsString, path::PathBuf, sync::Arc};

macro_rules! ffarg {
    ($args:ident, $arg:expr) => {{
        $args.push(::std::ffi::OsString::from($arg));
    }};
    ($args:ident, $arg:expr, $value:expr) => {{
        $args.push(::std::ffi::OsString::from($arg));
        $args.push(::std::ffi::OsString::from($value));
    }};
}

pub mod audio;
pub use audio::AudioAacProfile;

pub mod subtitle;
pub use subtitle::SubtitleWebVttProfile;

pub mod video;
pub use video::VideoCopyProfile;
pub use video::VideoH264Profile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProfileType {
    Copy,
    Transcode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentLayout {
    Keyframe,
    Fixed,
    Single,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaylistKind {
    Fmp4,
    WebVtt,
}

pub struct ProfileContext {
    pub input: PathBuf,
    pub stream: StreamDescriptor,
    pub stream_info: Option<StreamInfo>,
    pub keyframes: Option<Arc<Vec<i64>>>,
}

pub trait Profile: Send + Sync {
    fn display_name(&self) -> &'static str;
    fn id_name(&self) -> &'static str;
    fn profile_type(&self) -> ProfileType;
    fn segment_layout(&self) -> SegmentLayout;
    fn playlist_kind(&self) -> PlaylistKind;
    fn stream_type(&self) -> StreamType;
    fn supports_stream(&self, ctx: &ProfileContext) -> bool;
    fn build_args(
        &self,
        ctx: &ProfileContext,
        start_segment: i64,
        start_seconds: f64,
        hls_cuts: &str,
    ) -> Vec<OsString>;

    fn init_segment_name(&self) -> Option<&'static str> {
        match self.playlist_kind() {
            PlaylistKind::Fmp4 => Some("init.mp4"),
            PlaylistKind::WebVtt => None,
        }
    }

    fn segment_file_extension(&self) -> &'static str {
        match self.playlist_kind() {
            PlaylistKind::Fmp4 => "m4s",
            PlaylistKind::WebVtt => "vtt",
        }
    }

    fn segment_content_type(&self) -> &'static str {
        match self.playlist_kind() {
            PlaylistKind::Fmp4 => "video/mp4",
            PlaylistKind::WebVtt => "text/vtt; charset=utf-8",
        }
    }
}
