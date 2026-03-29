use crate::content_update::CONTENT_UPDATE;
use crate::entities::jobs::JobKind;
use lazy_static::lazy_static;
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

lazy_static! {
    pub static ref ACTIVITY_REGISTRY: ActivityRegistry = ActivityRegistry::new();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ActivityKind {
    LibraryScan,
    FileGenerateTimelinePreview,
    FileGenerateThumbnail,
    FileExtractFfprobe,
    FileExtractKeyframes,
    AssetDownload,
    AssetGenerateThumbhash,
    NodeGenerateIntroSegments,
    NodeMatchMetadataRoot,
    NodeMatchMetadataGroups,
}

impl ActivityKind {
    pub const fn title(self) -> &'static str {
        match self {
            ActivityKind::LibraryScan => "Library Scan",
            ActivityKind::FileGenerateTimelinePreview => "Timeline Preview Generation",
            ActivityKind::FileGenerateThumbnail => "Thumbnail Generation",
            ActivityKind::FileExtractFfprobe => "Probe Files",
            ActivityKind::FileExtractKeyframes => "Keyframe Extraction",
            ActivityKind::AssetDownload => "Asset Download",
            ActivityKind::AssetGenerateThumbhash => "Asset Preview Generation",
            ActivityKind::NodeGenerateIntroSegments => "Intro Detection",
            ActivityKind::NodeMatchMetadataRoot => "Match Root Metadata",
            ActivityKind::NodeMatchMetadataGroups => "Match Grouped Node Metadata",
        }
    }

    pub const fn task_type(self) -> &'static str {
        match self {
            ActivityKind::LibraryScan => "library_scan",
            ActivityKind::FileGenerateTimelinePreview => "timeline_preview",
            ActivityKind::FileGenerateThumbnail => "thumbnail",
            ActivityKind::FileExtractFfprobe => "ffprobe",
            ActivityKind::FileExtractKeyframes => "keyframes",
            ActivityKind::AssetDownload => "download",
            ActivityKind::AssetGenerateThumbhash => "thumbhash",
            ActivityKind::NodeGenerateIntroSegments => "intro_segments",
            ActivityKind::NodeMatchMetadataRoot => "metadata_match_root",
            ActivityKind::NodeMatchMetadataGroups => "metadata_match_groups",
        }
    }
}

impl From<JobKind> for ActivityKind {
    fn from(value: JobKind) -> Self {
        match value {
            JobKind::FileGenerateTimelinePreview => ActivityKind::FileGenerateTimelinePreview,
            JobKind::FileGenerateThumbnail => ActivityKind::FileGenerateThumbnail,
            JobKind::FileExtractFfprobe => ActivityKind::FileExtractFfprobe,
            JobKind::FileExtractKeyframes => ActivityKind::FileExtractKeyframes,
            JobKind::AssetDownload => ActivityKind::AssetDownload,
            JobKind::AssetGenerateThumbhash => ActivityKind::AssetGenerateThumbhash,
            JobKind::NodeGenerateIntroSegments => ActivityKind::NodeGenerateIntroSegments,
            JobKind::NodeMatchMetadataRoot => ActivityKind::NodeMatchMetadataRoot,
            JobKind::NodeMatchMetadataGroups => ActivityKind::NodeMatchMetadataGroups,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivitySnapshot {
    pub kind: ActivityKind,
    pub task_type: String,
    pub title: String,
    pub current: Option<i64>,
    pub total: Option<i64>,
    pub progress_percent: Option<f64>,
}

#[derive(Clone, Copy, Debug, Default)]
struct ActivityState {
    progress: Option<i64>,
    total: Option<i64>,
}

pub struct ActivityRegistry {
    next_id: AtomicU64,
    handles: Mutex<HashMap<u64, RegisteredActivity>>,
}

#[derive(Clone, Copy, Debug)]
struct RegisteredActivity {
    kind: ActivityKind,
    state: ActivityState,
}

impl ActivityRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            handles: Mutex::new(HashMap::new()),
        }
    }

    fn register(&self, kind: ActivityKind) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.handles.lock().unwrap().insert(
            id,
            RegisteredActivity {
                kind,
                state: ActivityState::default(),
            },
        );
        CONTENT_UPDATE.emit();
        id
    }

    fn update_total(&self, id: u64, total: i64) {
        let mut handles = self.handles.lock().unwrap();
        let Some(activity) = handles.get_mut(&id) else {
            return;
        };
        activity.state.total = Some(total.max(0));
        drop(handles);
        CONTENT_UPDATE.emit();
    }

    fn update_progress(&self, id: u64, progress: i64) {
        let mut handles = self.handles.lock().unwrap();
        let Some(activity) = handles.get_mut(&id) else {
            return;
        };
        activity.state.progress = Some(progress.max(0));
        drop(handles);
        CONTENT_UPDATE.emit();
    }

    fn unregister(&self, id: u64) {
        if self.handles.lock().unwrap().remove(&id).is_some() {
            CONTENT_UPDATE.emit();
        }
    }

    pub fn snapshot(&self) -> Vec<ActivitySnapshot> {
        let handles = self.handles.lock().unwrap();
        let mut by_kind: BTreeMap<ActivityKind, Vec<ActivityState>> = BTreeMap::new();

        for activity in handles.values() {
            by_kind
                .entry(activity.kind)
                .or_default()
                .push(activity.state);
        }

        by_kind
            .into_iter()
            .map(|(kind, states)| aggregate_activity(kind, &states))
            .collect()
    }
}

pub struct ActivityHandle {
    id: u64,
}

impl ActivityHandle {
    pub fn new(kind: impl Into<ActivityKind>) -> Self {
        Self {
            id: ACTIVITY_REGISTRY.register(kind.into()),
        }
    }

    pub fn set_total(&mut self, total: i64) {
        ACTIVITY_REGISTRY.update_total(self.id, total);
    }

    pub fn set_progress(&mut self, progress: i64) {
        ACTIVITY_REGISTRY.update_progress(self.id, progress);
    }
}

impl Drop for ActivityHandle {
    fn drop(&mut self) {
        ACTIVITY_REGISTRY.unregister(self.id);
    }
}

// We only surface numeric progress when at least one running task has declared a usable scale.
// Otherwise the dashboard should render an indeterminate spinner for that activity kind.
fn aggregate_activity(kind: ActivityKind, states: &[ActivityState]) -> ActivitySnapshot {
    let progress_states = states
        .iter()
        .filter_map(|state| match (state.progress, state.total) {
            (Some(progress), Some(total)) => Some((progress, total)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let (current, total, progress_percent) = if progress_states.is_empty() {
        (None, None, None)
    } else {
        let current = progress_states
            .iter()
            .map(|(progress, _)| *progress)
            .sum::<i64>();
        let total = progress_states.iter().map(|(_, total)| *total).sum::<i64>();
        let progress_percent = if total > 0 {
            Some((current as f64 / total as f64).clamp(0.0, 1.0))
        } else {
            None
        };

        (Some(current), Some(total), progress_percent)
    };

    ActivitySnapshot {
        kind,
        task_type: kind.task_type().to_owned(),
        title: kind.title().to_owned(),
        current,
        total,
        progress_percent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_indeterminate_handles() {
        let snapshot = aggregate_activity(
            ActivityKind::LibraryScan,
            &[ActivityState::default(), ActivityState::default()],
        );

        assert_eq!(snapshot.current, None);
        assert_eq!(snapshot.total, None);
        assert_eq!(snapshot.progress_percent, None);
    }

    #[test]
    fn aggregates_progress_from_reported_handles_only() {
        let snapshot = aggregate_activity(
            ActivityKind::FileGenerateTimelinePreview,
            &[
                ActivityState {
                    progress: Some(3),
                    total: Some(10),
                },
                ActivityState::default(),
                ActivityState {
                    progress: Some(7),
                    total: Some(10),
                },
            ],
        );

        assert_eq!(snapshot.current, Some(10));
        assert_eq!(snapshot.total, Some(20));
        assert_eq!(snapshot.progress_percent, Some(0.5));
    }
}
