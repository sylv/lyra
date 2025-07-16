use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, sqlx::Type, specta::Type, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum MediaType {
    Movie,
    Show,
    Season,
    Episode,
}

impl MediaType {
    pub fn as_int(&self) -> i32 {
        match self {
            MediaType::Movie => 0,
            MediaType::Show => 1,
            MediaType::Season => 2,
            MediaType::Episode => 3,
        }
    }

    pub fn from_int(value: i32) -> Option<Self> {
        match value {
            0 => Some(MediaType::Movie),
            1 => Some(MediaType::Show),
            2 => Some(MediaType::Season),
            3 => Some(MediaType::Episode),
            _ => None,
        }
    }
}

#[derive(Debug, specta::Type, sqlx::FromRow, Serialize)]
pub struct Media {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub background_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub parent_id: Option<i64>,
    pub media_type: MediaType,
    pub tmdb_parent_id: i64,
    pub tmdb_item_id: Option<i64>,
    pub rating: Option<f64>,
    pub release_date: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
}

impl Media {
    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Media>, sqlx::Error> {
        let media = sqlx::query_as!(
            Media,
            "SELECT id, name, description, poster_url, background_url, thumbnail_url, parent_id, media_type as \"media_type: MediaType\", tmdb_parent_id, tmdb_item_id, rating, release_date, runtime_minutes, season_number, episode_number 
            FROM media WHERE id = ?",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(media)
    }

    pub async fn find_by_parent_id(
        pool: &SqlitePool,
        parent_id: Option<i64>,
    ) -> Result<Vec<Media>, sqlx::Error> {
        let media = if let Some(parent_id) = parent_id {
            sqlx::query_as!(
                Media,
                "SELECT id, name, description, poster_url, background_url, thumbnail_url, parent_id, media_type as \"media_type: MediaType\", tmdb_parent_id, tmdb_item_id, rating, release_date, runtime_minutes, season_number, episode_number 
                 FROM media 
                 WHERE parent_id = ?",
                parent_id
            )
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as!(
                Media,
                "SELECT id, name, description, poster_url, background_url, thumbnail_url, parent_id, media_type as \"media_type: MediaType\", tmdb_parent_id, tmdb_item_id, rating, release_date, runtime_minutes, season_number, episode_number 
                 FROM media 
                 WHERE parent_id IS NULL",
            )
            .fetch_all(pool)
            .await?
        };

        Ok(media)
    }

    pub async fn find_default_connections(
        pool: &SqlitePool,
        media_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, crate::entities::file::File>, sqlx::Error> {
        if media_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        // todo: this is kinda ass
        let placeholders = media_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            r#"
            WITH RECURSIVE media_descendants AS (
                -- Base case: get all the media items we're interested in
                SELECT 
                    m.id as root_id,
                    m.id,
                    m.parent_id,
                    m.season_number,
                    m.episode_number,
                    m.media_type,
                    m.release_date
                FROM media m
                WHERE m.id IN ({})
                
                UNION ALL
                
                -- Recursive case: get all descendants
                SELECT 
                    md.root_id,
                    m.id,
                    m.parent_id,
                    m.season_number,
                    m.episode_number,
                    m.media_type,
                    m.release_date
                FROM media m
                JOIN media_descendants md ON m.parent_id = md.id
            ),
            media_with_files AS (
                SELECT 
                    md.root_id,
                    md.id as media_id,
                    md.season_number,
                    md.episode_number,
                    md.media_type,
                    md.release_date,
                    f.id as file_id,
                    f.backend_name,
                    f.key,
                    f.pending_auto_match,
                    f.unavailable_since,
                    f.edition_name
                FROM media_descendants md
                JOIN media_connection mc ON md.id = mc.media_id
                JOIN file f ON mc.file_id = f.id
                WHERE f.unavailable_since IS NULL
            ),
            ranked_files AS (
                SELECT 
                    *,
                    ROW_NUMBER() OVER (
                        PARTITION BY root_id 
                        ORDER BY 
                            -- Prefer episodes over movies/shows/seasons
                            CASE WHEN media_type = 3 THEN 0 ELSE 1 END,
                            -- Then order by season number (nulls last)
                            season_number NULLS LAST,
                            -- Then by episode number (nulls last)  
                            episode_number NULLS LAST,
                            -- Then by release date (nulls last)
                            release_date NULLS LAST,
                            -- Finally by media_id for consistency
                            media_id
                    ) as rn
                FROM media_with_files
            )
            SELECT 
                root_id,
                file_id,
                backend_name,
                key,
                pending_auto_match,
                unavailable_since,
                edition_name
            FROM ranked_files 
            WHERE rn = 1
            "#,
            placeholders
        );

        let mut query_builder = sqlx::query(&query);
        for &media_id in media_ids {
            query_builder = query_builder.bind(media_id);
        }

        let rows = query_builder.fetch_all(pool).await?;

        let mut result = std::collections::HashMap::new();
        for row in rows {
            let root_id: i64 = row.get("root_id");
            let file = crate::entities::file::File {
                id: row.get("file_id"),
                backend_name: row.get("backend_name"),
                key: row.get("key"),
                pending_auto_match: row.get("pending_auto_match"),
                unavailable_since: row.get("unavailable_since"),
                edition_name: row.get("edition_name"),
            };
            result.insert(root_id, file);
        }

        Ok(result)
    }
}

pub struct UpsertMedia {
    pub name: String,
    pub media_type: MediaType,
    pub description: Option<String>,
    pub poster_url: Option<String>,
    pub background_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub parent_id: Option<i64>,
    pub tmdb_parent_id: i64,
    pub tmdb_item_id: Option<i64>,
    pub rating: Option<f64>,
    pub release_date: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
}

impl UpsertMedia {
    pub fn new(name: String, media_type: MediaType, tmdb_parent_id: i64) -> Self {
        Self {
            name,
            media_type,
            description: None,
            poster_url: None,
            background_url: None,
            thumbnail_url: None,
            parent_id: None,
            tmdb_parent_id,
            tmdb_item_id: None,
            rating: None,
            release_date: None,
            runtime_minutes: None,
            season_number: None,
            episode_number: None,
        }
    }

    pub async fn upsert(self, pool: &SqlitePool) -> Result<Media, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let media_type = self.media_type.as_int();
        let existing_id = match self.media_type {
            MediaType::Movie | MediaType::Show => {
                sqlx::query_scalar!(
                    "SELECT id FROM media WHERE tmdb_parent_id = ? AND media_type = ?",
                    self.tmdb_parent_id,
                    media_type
                )
                .fetch_optional(tx.as_mut())
                .await?
            }
            MediaType::Season => {
                let season_number = self
                    .season_number
                    .expect("season_number is required for season media type");
                sqlx::query_scalar!(
                        "SELECT id FROM media WHERE tmdb_parent_id = ? AND season_number = ? AND media_type = ?",
                        self.tmdb_parent_id,
                        season_number,
                        media_type
                    )
                    .fetch_optional(tx.as_mut())
                    .await?
            }
            MediaType::Episode => {
                let season_number = self
                    .season_number
                    .expect("season_number is required for episode media type");
                let episode_number = self
                    .episode_number
                    .expect("episode_number is required for episode media type");
                sqlx::query_scalar!(
                        "SELECT id FROM media WHERE tmdb_parent_id = ? AND season_number = ? AND episode_number = ? AND media_type = ?",
                        self.tmdb_parent_id,
                        season_number,
                        episode_number,
                        media_type
                    )
                    .fetch_optional(tx.as_mut())
                    .await?
            }
        };

        let media = if let Some(id) = existing_id {
            sqlx::query_as!(
                Media,
                "UPDATE media SET
                    name = ?,
                    description = ?,
                    poster_url = ?,
                    background_url = ?,
                    thumbnail_url = ?,
                    parent_id = ?,
                    tmdb_parent_id = ?,
                    tmdb_item_id = ?,
                    rating = ?,
                    release_date = ?,
                    runtime_minutes = ?,
                    season_number = ?,
                    episode_number = ?
                WHERE id = ?
                RETURNING
                    id,
                    name,
                    description,
                    poster_url,
                    background_url,
                    thumbnail_url,
                    parent_id,
                    media_type as \"media_type: MediaType\",
                    tmdb_parent_id,
                    tmdb_item_id,
                    rating,
                    release_date,
                    runtime_minutes,
                    season_number,
                    episode_number
                ",
                self.name,
                self.description,
                self.poster_url,
                self.background_url,
                self.thumbnail_url,
                self.parent_id,
                self.tmdb_parent_id,
                self.tmdb_item_id,
                self.rating,
                self.release_date,
                self.runtime_minutes,
                self.season_number,
                self.episode_number,
                id
            )
            .fetch_one(tx.as_mut())
            .await?
        } else {
            sqlx::query_as!(
                Media,
                "INSERT INTO media (
                    name,
                    media_type,
                    description,
                    poster_url,
                    background_url,
                    thumbnail_url,
                    parent_id,
                    tmdb_parent_id,
                    tmdb_item_id,
                    rating,
                    release_date,
                    runtime_minutes,
                    season_number,
                    episode_number
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING
                    id,
                    name,
                    description,
                    poster_url,
                    background_url,
                    thumbnail_url,
                    parent_id,
                    media_type as \"media_type: MediaType\",
                    tmdb_parent_id,
                    tmdb_item_id,
                    rating,
                    release_date,
                    runtime_minutes,
                    season_number,
                    episode_number
                ",
                self.name,
                media_type,
                self.description,
                self.poster_url,
                self.background_url,
                self.thumbnail_url,
                self.parent_id,
                self.tmdb_parent_id,
                self.tmdb_item_id,
                self.rating,
                self.release_date,
                self.runtime_minutes,
                self.season_number,
                self.episode_number,
            )
            .fetch_one(tx.as_mut())
            .await?
        };

        tx.commit().await?;
        Ok(media)
    }
}
