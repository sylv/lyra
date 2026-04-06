use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_bit_rate: Option<u64>,
    pub streams: Vec<Stream>,
}

impl ProbeData {
    pub fn get_video_stream(&self) -> Option<&Stream> {
        let mut best = None;
        for stream in &self.streams {
            if stream.kind() != StreamKind::Video {
                continue;
            }

            match best {
                None => best = Some(stream),
                Some(current_best) => {
                    // if current is default but best isn't, pick current
                    if stream.disposition.contains(StreamDisposition::DEFAULT)
                        && !current_best
                            .disposition
                            .contains(StreamDisposition::DEFAULT)
                    {
                        best = Some(stream);
                    }
                    // if current is lower index than best, pick current
                    else if stream.index < current_best.index {
                        best = Some(stream);
                    }
                }
            }
        }

        best
    }

    pub fn get_audio_stream(&self) -> Option<&Stream> {
        let mut best = None;
        for stream in &self.streams {
            if stream.kind() != StreamKind::Audio {
                continue;
            }

            match best {
                None => best = Some(stream),
                Some(current_best) => {
                    if stream.disposition.contains(StreamDisposition::DEFAULT)
                        && !current_best
                            .disposition
                            .contains(StreamDisposition::DEFAULT)
                    {
                        best = Some(stream);
                    } else if stream.index < current_best.index {
                        best = Some(stream);
                    }
                }
            }
        }

        best
    }

    pub fn has_subtitles(&self) -> bool {
        self.streams
            .iter()
            .any(|stream| stream.kind() == StreamKind::Subtitle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamKind {
    Video,
    Audio,
    Subtitle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Codec {
    VideoH264,
    VideoH265,
    VideoAv1,
    AudioAac,
    SubtitleAss,
    SubtitleSubRip,
    SubtitleMovText,
    SubtitleText,
    SubtitleTtml,
    SubtitleWebVtt,
    SubtitlePgs,
    SubtitleVobSub,
    Unknown(String),
}

impl Codec {
    pub fn from_str(value: &str) -> Self {
        let value = value.to_ascii_lowercase();
        match value.as_str() {
            "av1" => Self::VideoAv1,
            "h264" | "avc" => Self::VideoH264,
            "h265" | "hevc" => Self::VideoH265,
            "aac" => Self::AudioAac,
            "ass" | "ssa" => Self::SubtitleAss,
            "mov_text" => Self::SubtitleMovText,
            "subrip" | "srt" => Self::SubtitleSubRip,
            "text" => Self::SubtitleText,
            "ttml" => Self::SubtitleTtml,
            "webvtt" => Self::SubtitleWebVtt,
            "hdmv_pgs_subtitle" | "pgs" => Self::SubtitlePgs,
            "dvd_subtitle" | "vobsub" | "dvdsub" => Self::SubtitleVobSub,
            _ => Self::Unknown(value),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::VideoH264 => "h264",
            Self::VideoH265 => "h265",
            Self::VideoAv1 => "av1",
            Self::AudioAac => "aac",
            Self::SubtitleAss => "ass",
            Self::SubtitleSubRip => "subrip",
            Self::SubtitleMovText => "mov_text",
            Self::SubtitleText => "text",
            Self::SubtitleTtml => "ttml",
            Self::SubtitleWebVtt => "webvtt",
            Self::SubtitlePgs => "hdmv_pgs_subtitle",
            Self::SubtitleVobSub => "dvd_subtitle",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl Serialize for Codec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Codec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from_str(&value))
    }
}

impl fmt::Display for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub index: u32,
    #[serde(rename = "codec_name")]
    pub codec: Codec,
    /// Cleaned up display name, e.g. "English (Forced)" vs "eng_forced"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Raw title from ffprobe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_bcp47: Option<String>,
    #[serde(with = "disposition_as_bits")]
    pub disposition: StreamDisposition,
    #[serde(flatten)]
    pub details: StreamDetails,
}

impl Stream {
    pub fn kind(&self) -> StreamKind {
        match self.details {
            StreamDetails::Video { .. } => StreamKind::Video,
            StreamDetails::Audio { .. } => StreamKind::Audio,
            StreamDetails::Subtitle { .. } => StreamKind::Subtitle,
        }
    }

    pub fn is_forced(&self) -> bool {
        self.disposition.contains(StreamDisposition::FORCED)
    }

    pub fn is_hearing_impaired(&self) -> bool {
        self.disposition
            .contains(StreamDisposition::HEARING_IMPAIRED)
    }

    pub fn is_commentary(&self) -> bool {
        self.disposition.contains(StreamDisposition::COMMENTARY)
    }

    pub fn width(&self) -> Option<u32> {
        match &self.details {
            StreamDetails::Video { width, .. } => Some(*width),
            _ => None,
        }
    }

    pub fn height(&self) -> Option<u32> {
        match &self.details {
            StreamDetails::Video { height, .. } => Some(*height),
            _ => None,
        }
    }

    pub fn frame_rate(&self) -> Option<f32> {
        match &self.details {
            StreamDetails::Video { frame_rate, .. } => *frame_rate,
            _ => None,
        }
    }

    pub fn time_base(&self) -> Option<(i64, i64)> {
        match &self.details {
            StreamDetails::Video {
                time_base_num,
                time_base_den,
                ..
            } => Some((*time_base_num, *time_base_den)),
            _ => None,
        }
    }

    pub fn channels(&self) -> Option<u16> {
        match &self.details {
            StreamDetails::Audio { channels, .. } => Some(*channels),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StreamDetails {
    Video {
        width: u32,
        height: u32,
        time_base_num: i64,
        time_base_den: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        frame_rate: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bit_depth: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hdr_format: Option<HDRFormat>,
    },
    Audio {
        channels: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        sample_rate: Option<u32>,
    },
    Subtitle {
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<SubtitleFormat>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HDRFormat {
    Hdr10,
    DolbyVision,
    Hlg,
    Hdr10Plus,
    Unknown(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtitleFormat {
    Srt,
    Ass,
    WebVtt,
    Pgs,
    VobSub,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct StreamDisposition: u16 {
        const DEFAULT           = 1 << 0;
        const FORCED            = 1 << 1;
        const COMMENTARY        = 1 << 2;
        const HEARING_IMPAIRED  = 1 << 3;
        const VISUAL_IMPAIRED   = 1 << 4;
        const ORIGINAL          = 1 << 5;
        const DUBBED            = 1 << 6;
    }
}

mod disposition_as_bits {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &StreamDisposition, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(value.bits())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StreamDisposition, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u16::deserialize(deserializer)?;
        StreamDisposition::from_bits(bits)
            .ok_or_else(|| serde::de::Error::custom("invalid stream disposition bits"))
    }
}

#[cfg(test)]
mod tests {
    use super::Codec;

    #[test]
    fn codec_parsing_normalizes_aliases_and_case() {
        assert_eq!(Codec::from_str("hevc"), Codec::VideoH265);
        assert_eq!(Codec::from_str("H264"), Codec::VideoH264);
        assert_eq!(Codec::from_str("SSA"), Codec::SubtitleAss);
        assert_eq!(
            Codec::from_str("SomethingCustom"),
            Codec::Unknown("somethingcustom".to_string())
        );
    }
}
