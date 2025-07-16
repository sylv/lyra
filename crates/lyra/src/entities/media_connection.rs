use sqlx::SqlitePool;

pub struct MediaConnection {
    pub media_id: i64,
    pub file_id: i64,
}

impl MediaConnection {
    pub async fn create(pool: &SqlitePool, media_id: i64, file_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT OR IGNORE INTO media_connection (media_id, file_id) VALUES (?, ?)",
            media_id,
            file_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_file_id(
        pool: &SqlitePool,
        file_id: i64,
    ) -> Result<Vec<MediaConnection>, sqlx::Error> {
        let connections = sqlx::query_as!(
            MediaConnection,
            "SELECT media_id, file_id FROM media_connection WHERE file_id = ?",
            file_id
        )
        .fetch_all(pool)
        .await?;
        Ok(connections)
    }

    pub async fn find_by_media_id(
        pool: &SqlitePool,
        media_id: i64,
    ) -> Result<Vec<MediaConnection>, sqlx::Error> {
        let connections = sqlx::query_as!(
            MediaConnection,
            "SELECT media_id, file_id FROM media_connection WHERE media_id = ?",
            media_id
        )
        .fetch_all(pool)
        .await?;
        Ok(connections)
    }
}
