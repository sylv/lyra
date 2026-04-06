mod detect;
mod fingerprint;
mod generate;

pub use detect::{FileIntroDetection, IntroRange, detect_intros};
pub use fingerprint::Fingerprint;
pub use generate::fingerprint;
use rusty_chromaprint::Configuration;

pub(crate) const AUDIO_FINGERPRINT_VERSION: u32 = 1;
pub(crate) const AUDIO_FINGERPRINT_CACHE_MAGIC: [u8; 4] = *b"LAFP";
pub(crate) const AUDIO_FINGERPRINT_CACHE_SCHEMA_VERSION: u32 = 1;
pub(crate) const FINGERPRINT_SCAN_RATIO: f64 = 0.40;
pub(crate) const FINGERPRINT_SAMPLE_RATE: u32 = 48_000;
pub(crate) const FINGERPRINT_CHANNELS: u32 = 2;
pub(crate) const MIN_MATCH_DURATION_SECONDS: f32 = 8.0;
pub(crate) const MAX_MATCH_DURATION_SECONDS: f32 = 180.0;
pub(crate) const MERGE_SEGMENT_GAP_SECONDS: f32 = 2.0;
pub(crate) const MIN_INTRO_EPISODE_COUNT: usize = 3;

pub(crate) fn chromaprint_config() -> Configuration {
    Configuration::preset_test1()
        .with_id(50)
        .with_removed_silence(50)
}
