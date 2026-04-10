use crate::{
    profiles::{Profile, ProfileArgsPosition, ProfileContext},
    types::Compatibility,
};
use lyra_probe::{Codec, Stream, StreamKind};
use std::ffi::OsString;

macro_rules! ffarg {
    ($args:ident, $arg:expr) => {{
        $args.push(::std::ffi::OsString::from($arg));
    }};
    ($args:ident, $arg:expr, $value:expr) => {{
        $args.push(::std::ffi::OsString::from($arg));
        $args.push(::std::ffi::OsString::from($value));
    }};
}

pub static VIDEO_COPY_PROFILE: VideoCopyProfile = VideoCopyProfile;
pub static VIDEO_H264_PROFILE: VideoH264Profile = VideoH264Profile;

pub struct VideoCopyProfile;

impl VideoCopyProfile {
    pub const ID: &'static str = "copy";
}

impl Profile for VideoCopyProfile {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn compatible_with(&self, stream: &Stream) -> Option<Compatibility> {
        if stream.kind() != StreamKind::Video {
            return None;
        }

        matches!(
            stream.codec,
            Codec::VideoH264 | Codec::VideoH265 | Codec::VideoAv1
        )
        .then_some(Compatibility::KeyframeAligned)
    }

    fn append_args(
        &self,
        args: &mut Vec<OsString>,
        context: &ProfileContext<'_>,
    ) -> anyhow::Result<()> {
        match context.position {
            ProfileArgsPosition::BeforeInput => {
                if let Some(start_seconds) = context.start_seconds() {
                    ffarg!(args, "-ss", format!("{start_seconds:.6}"));
                    ffarg!(args, "-noaccurate_seek");
                }
            }
            ProfileArgsPosition::AfterInput => {
                anyhow::ensure!(
                    context.keyframes.is_some(),
                    "copy profile requires keyframe metadata for segment alignment"
                );
                ffarg!(args, "-codec:v", "copy");
            }
        }
        Ok(())
    }
}

pub struct VideoH264Profile;

impl VideoH264Profile {
    pub const ID: &'static str = "h264";
}

impl Profile for VideoH264Profile {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn compatible_with(&self, stream: &Stream) -> Option<Compatibility> {
        (stream.kind() == StreamKind::Video).then_some(Compatibility::Fixed)
    }

    fn append_args(
        &self,
        args: &mut Vec<OsString>,
        context: &ProfileContext<'_>,
    ) -> anyhow::Result<()> {
        match context.position {
            ProfileArgsPosition::BeforeInput => {
                if let Some(start_seconds) = context.start_seconds() {
                    ffarg!(args, "-ss", format!("{start_seconds:.6}"));
                }
            }
            ProfileArgsPosition::AfterInput => {
                ffarg!(args, "-codec:v", "libx264");
                ffarg!(args, "-preset", "veryfast");
                ffarg!(args, "-force_key_frames", "expr:gte(t,n_forced*6)");
            }
        }
        Ok(())
    }
}
