use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, specta::Type, Serialize)]
pub struct File {
    pub id: i64,
    pub backend_name: String,
    pub key: String,
    pub pending_auto_match: i64,
    pub unavailable_since: Option<i64>,
    pub edition_name: Option<String>,
}

impl File {
    pub async fn find_by_media_id(
        pool: &SqlitePool,
        media_id: i64,
    ) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as!(
            File,
            "SELECT id, backend_name, key, pending_auto_match, unavailable_since, edition_name
            FROM file 
            WHERE id IN (SELECT file_id FROM media_connection WHERE media_id = ?)",
            media_id
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<File>, sqlx::Error> {
        let file = sqlx::query_as!(
            File,
            "SELECT id, backend_name, key, pending_auto_match, unavailable_since, edition_name
            FROM file WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(file)
    }
}
