use crate::{
    profiles::{Profile, ProfileArgsPosition, ProfileContext},
    types::Compatibility,
};
use lyra_probe::{Stream, StreamKind};
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

pub static AUDIO_AAC_PROFILE: AudioAacProfile = AudioAacProfile;

pub struct AudioAacProfile;

impl AudioAacProfile {
    pub const ID: &'static str = "aac";
}

impl Profile for AudioAacProfile {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn compatible_with(&self, stream: &Stream) -> Option<Compatibility> {
        (stream.kind() == StreamKind::Audio).then_some(Compatibility::Fixed)
    }

    fn append_args(
        &self,
        args: &mut Vec<OsString>,
        context: &ProfileContext<'_>,
    ) -> anyhow::Result<()> {
        if context.position == ProfileArgsPosition::BeforeInput {
            return Ok(());
        }

        ffarg!(args, "-codec:a", "aac");
        ffarg!(args, "-profile:a", "aac_low");
        ffarg!(args, "-ac", "2");
        ffarg!(args, "-b:a", "160k");

        if context.stream.channels().is_some_and(|channels| channels != 2) {
            ffarg!(
                args,
                "-af",
                "pan=stereo|FL=0.5*FC+0.707*FL+0.707*BL+0.5*LFE|FR=0.5*FC+0.707*FR+0.707*BR+0.5*LFE"
            );
        }

        Ok(())
    }
}
