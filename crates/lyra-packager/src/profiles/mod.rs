use crate::types::Compatibility;
use lyra_probe::{Stream, VideoKeyframes};
use std::{ffi::OsString, time::Duration};

pub mod audio;
pub mod video;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileArgsPosition {
    BeforeInput,
    AfterInput,
}

pub struct ProfileContext<'a> {
    pub stream: &'a Stream,
    pub keyframes: Option<&'a VideoKeyframes>,
    pub segment_index: usize,
    pub target_segment_duration: Duration,
    pub compatibility: Compatibility,
    pub position: ProfileArgsPosition,
}

impl ProfileContext<'_> {
    pub fn start_seconds(&self) -> Option<f64> {
        if self.segment_index == 0 {
            return None;
        }

        Some(match self.compatibility {
            Compatibility::KeyframeAligned => {
                let keyframes = self
                    .keyframes
                    .expect("keyframe-aligned profiles require keyframes");
                keyframes.pts_to_seconds(
                    keyframes.segment_start_pts_at(self.segment_index, self.target_segment_duration),
                )
            }
            Compatibility::Fixed => {
                self.segment_index as f64 * self.target_segment_duration.as_secs_f64()
            }
        })
    }
}

pub trait Profile: Send + Sync {
    fn id(&self) -> &'static str;
    fn compatible_with(&self, stream: &Stream) -> Option<Compatibility>;
    fn append_args(&self, args: &mut Vec<OsString>, context: &ProfileContext<'_>)
        -> anyhow::Result<()>;
}

pub fn video_profile(id: &str) -> Option<&'static dyn Profile> {
    match id {
        video::VideoCopyProfile::ID => Some(&video::VIDEO_COPY_PROFILE),
        video::VideoH264Profile::ID => Some(&video::VIDEO_H264_PROFILE),
        _ => None,
    }
}

pub fn audio_profile(id: &str) -> Option<&'static dyn Profile> {
    match id {
        audio::AudioAacProfile::ID => Some(&audio::AUDIO_AAC_PROFILE),
        _ => None,
    }
}
