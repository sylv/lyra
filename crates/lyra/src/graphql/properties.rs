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
pub enum SubtitleSource {
    Extracted,
    Converted,
    Ocr,
    Generated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Enum)]
pub enum SubtitleKind {
    Srt,
    Vtt,
    Ass,
    MovText,
    Text,
    Ttml,
    Pgs,
    VobSub,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct SubtitleTrack {
    pub id: String,
    pub stream_index: i32,
    pub kind: SubtitleKind,
    pub source: SubtitleSource,
    pub label: String,
    pub language: Option<String>,
    pub dispositions: Vec<String>,
    pub asset: Asset,
    pub derived_from_subtitle_id: Option<String>,
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
pub struct PlaybackOptions {
    pub video_renditions: Vec<VideoRenditionOption>,
    pub audio_tracks: Vec<AudioTrackOption>,
    pub subtitle_tracks: Vec<SubtitlePlaybackTrack>,
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

#[derive(Clone, Debug, SimpleObject)]
pub struct VideoRenditionOption {
    pub rendition_id: String,
    pub display_name: String,
    pub display_info: String,
    pub codec_tag: String,
    pub on_demand: bool,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct AudioTrackOption {
    pub stream_index: i32,
    pub display_name: String,
    pub language: Option<String>,
    pub recommended: bool,
    pub renditions: Vec<AudioRenditionOption>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct AudioRenditionOption {
    pub rendition_id: String,
    pub codec_name: String,
    pub bitrate: Option<i32>,
    pub channels: Option<i32>,
    pub sample_rate: Option<i32>,
    pub codec_tag: String,
    pub on_demand: bool,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct SubtitlePlaybackTrack {
    pub subtitle_id: String,
    pub stream_index: i32,
    pub display_name: String,
    pub language: Option<String>,
    pub recommended: bool,
    pub renditions: Vec<SubtitleRenditionOption>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct SubtitleRenditionOption {
    pub rendition_id: String,
    pub codec_name: String,
    pub on_demand: bool,
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
