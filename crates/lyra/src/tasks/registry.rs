use crate::tasks::{
    TaskManager, TaskRunner,
    tasks::{file_thumbnail::FileThumbnailTask, file_timeline_preview::FileTimelinePreviewTask},
};
use sea_orm::DatabaseConnection;

pub fn get_registered_tasks(pool: &DatabaseConnection) -> Vec<Box<dyn TaskRunner>> {
    vec![
        Box::new(TaskManager::<FileTimelinePreviewTask>::new(
            Box::new(FileTimelinePreviewTask),
            pool.clone(),
        )),
        Box::new(TaskManager::<FileThumbnailTask>::new(
            Box::new(FileThumbnailTask),
            pool.clone(),
        )),
    ]
}
