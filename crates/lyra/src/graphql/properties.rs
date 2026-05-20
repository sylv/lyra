use crate::entities::nodes;
use async_graphql::{Enum, SimpleObject};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum TrackDispositionPreference {
    Normal,
    Sdh,
    Commentary,
}

impl TrackDispositionPreference {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Normal" => Some(Self::Normal),
            "Sdh" => Some(Self::Sdh),
            "Commentary" => Some(Self::Commentary),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Sdh => "Sdh",
            Self::Commentary => "Commentary",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum PlaybackVideoProfileId {
    Copy,
    H264,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum PlaybackVideoCodec {
    H264,
    H265,
    Av1,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct PlaybackVideoTrack {
    pub source_track_id: String,
    pub display_name: String,
    pub autoselect: bool,
    pub renditions: Vec<PlaybackVideoRendition>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct PlaybackVideoRendition {
    pub pair_id: String,
    pub profile_id: PlaybackVideoProfileId,
    pub codec: PlaybackVideoCodec,
    pub display_info: String,
    pub codec_tag: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum PlaybackAudioProfileId {
    Aac,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum PlaybackAudioCodec {
    Aac,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct PlaybackAudioTrack {
    pub source_track_id: String,
    pub display_name: String,
    pub language_bcp47: Option<String>,
    pub autoselect: bool,
    pub renditions: Vec<PlaybackAudioRendition>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct PlaybackAudioRendition {
    pub pair_id: String,
    pub profile_id: PlaybackAudioProfileId,
    pub codec: PlaybackAudioCodec,
    pub display_info: String,
    pub codec_tag: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum PlaybackSubtitleKind {
    Forced,
    Subtitles,
    Captions,
    Commentary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum PlaybackSubtitleCodec {
    Vtt,
    Srt,
    Ass,
    MovText,
    Text,
    Ttml,
    Pgs,
    VobSub,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct PlaybackSubtitleTrack {
    pub source_track_id: String,
    pub display_name: String,
    pub language_bcp47: Option<String>,
    pub kind: PlaybackSubtitleKind,
    pub autoselect: bool,
    pub renditions: Vec<PlaybackSubtitleRendition>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct PlaybackSubtitleRendition {
    pub variant_id: Option<String>,
    pub signed_url: String,
    pub display_info: String,
    pub codec: PlaybackSubtitleCodec,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Asset {
    pub id: String,
    pub source_url: Option<String>,
    pub hash_sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub uncompressed_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub content_encoding: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    pub thumbhash: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct TimelinePreviewSheet {
    pub position_ms: i64,
    pub end_ms: i64,
    pub sheet_interval_ms: i64,
    pub sheet_gap_size: i64,
    pub asset: Asset,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct Playback {
    pub hls_url_template: String,
    pub video: Vec<PlaybackVideoTrack>,
    pub audio: Vec<PlaybackAudioTrack>,
    pub subtitles: Vec<PlaybackSubtitleTrack>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum MetadataStatus {
    Upcoming,
    Airing,
    Returning,
    Finished,
    Cancelled,
    InTheaters,
    Released,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct MetadataGenre {
    pub provider_id: String,
    pub external_id: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct ContentRating {
    pub country_code: String,
    pub rating: String,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub birthday: Option<String>,
    #[graphql(skip)]
    pub profile_asset_id: Option<String>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct CastMember {
    pub character_name: Option<String>,
    pub department: Option<String>,
    pub person: Person,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum FileSegmentKind {
    Intro,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct FileSegment {
    pub kind: FileSegmentKind,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct NodeProperties {
    pub display_name: String,
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub first_aired: Option<i64>,
    pub last_aired: Option<i64>,
    pub status: Option<MetadataStatus>,
    pub tagline: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    #[graphql(skip)]
    pub metadata_id: Option<String>,
    #[graphql(skip)]
    pub node_id: String,
    #[graphql(skip)]
    pub root_id: String,
    #[graphql(skip)]
    pub parent_id: Option<String>,
    #[graphql(skip)]
    pub kind: nodes::NodeKind,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct FileProbe {
    pub runtime_minutes: Option<i64>,
    pub duration_seconds: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub fps: Option<f64>,
    pub video_bitrate: Option<i64>,
    pub audio_bitrate: Option<i64>,
    pub audio_channels: Option<i64>,
    pub has_subtitles: bool,
}
