use lyra_marker::IntroRange;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredFileSegmentKind {
    Intro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredFileSegment {
    pub kind: StoredFileSegmentKind,
    pub start_ms: i64,
    pub end_ms: i64,
}

impl StoredFileSegment {
    pub const fn intro(start_ms: i64, end_ms: i64) -> Self {
        Self {
            kind: StoredFileSegmentKind::Intro,
            start_ms,
            end_ms,
        }
    }
}

pub fn intro_segment_from_range(range: IntroRange) -> Option<StoredFileSegment> {
    if !range.start_seconds.is_finite() || !range.end_seconds.is_finite() {
        return None;
    }

    let start_ms = (f64::from(range.start_seconds).max(0.0) * 1000.0).round() as i64;
    let end_ms = (f64::from(range.end_seconds).max(0.0) * 1000.0).round() as i64;
    if end_ms <= start_ms {
        return None;
    }

    Some(StoredFileSegment::intro(start_ms, end_ms))
}
