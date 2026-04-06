use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub index: u32,
    pub codec_name: String,
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
