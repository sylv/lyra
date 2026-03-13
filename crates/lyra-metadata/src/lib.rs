use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scored<T> {
    pub value: T,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootMatchHint {
    pub title: String,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesRootMatchRequest {
    pub hint: RootMatchHint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieRootMatchRequest {
    pub hint: RootMatchHint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesCandidate {
    pub tmdb_id: u64,
    pub name: String,
    pub first_air_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieCandidate {
    pub tmdb_id: u64,
    pub name: String,
    pub release_year: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageSet {
    pub poster_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub background_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMetadata {
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u64>,
    pub name: String,
    pub description: Option<String>,
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub images: ImageSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieMetadata {
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u64>,
    pub name: String,
    pub description: Option<String>,
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub images: ImageSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesItem {
    pub item_id: String,
    pub season_number: Option<i32>,
    pub episode_number: Option<i32>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesItemsRequest {
    pub root_id: String,
    pub candidate: SeriesCandidate,
    pub items: Vec<SeriesItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonMetadata {
    pub root_id: String,
    pub season_number: i32,
    pub name: String,
    pub description: Option<String>,
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub images: ImageSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMetadata {
    pub item_id: String,
    pub name: String,
    pub description: Option<String>,
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub released_at: Option<i64>,
    pub images: ImageSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesItemsResult {
    pub seasons: Vec<SeasonMetadata>,
    pub episodes: Vec<EpisodeMetadata>,
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    fn id(&self) -> &'static str;

    async fn match_series_root(
        &self,
        req: SeriesRootMatchRequest,
    ) -> Result<Vec<Scored<SeriesCandidate>>>;

    async fn lookup_series_metadata(&self, candidate: &SeriesCandidate) -> Result<SeriesMetadata>;

    async fn lookup_series_items(&self, req: SeriesItemsRequest) -> Result<SeriesItemsResult>;

    async fn match_movie_root(
        &self,
        req: MovieRootMatchRequest,
    ) -> Result<Vec<Scored<MovieCandidate>>>;

    async fn lookup_movie_metadata(&self, candidate: &MovieCandidate) -> Result<MovieMetadata>;
}
