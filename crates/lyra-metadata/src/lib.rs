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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataStatus {
    Upcoming,
    Airing,
    Returning,
    Finished,
    Cancelled,
    InTheaters,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataImageKind {
    Poster,
    Thumbnail,
    Backdrop,
    Logo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedMediaKind {
    Movie,
    Series,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataImage {
    pub kind: MetadataImageKind,
    pub url: String,
    pub language: Option<String>,
    pub vote_average: Option<f64>,
    pub vote_count: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub file_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageSet {
    pub posters: Vec<MetadataImage>,
    pub thumbnails: Vec<MetadataImage>,
    pub backdrops: Vec<MetadataImage>,
    pub logos: Vec<MetadataImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataGenre {
    pub provider_id: String,
    pub external_id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRating {
    pub country_code: String,
    pub rating: String,
    pub release_date: Option<i64>,
    pub release_type: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastCredit {
    pub provider_person_id: String,
    pub name: String,
    pub character_name: Option<String>,
    pub department: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonMetadata {
    pub provider_person_id: String,
    pub name: String,
    pub birthday: Option<String>,
    pub description: Option<String>,
    pub profile_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub media_kind: RecommendedMediaKind,
    pub tmdb_id: Option<u64>,
    pub imdb_id: Option<String>,
    pub name: String,
    pub first_aired: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMetadata {
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u64>,
    pub name: String,
    pub description: Option<String>,
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub first_aired: Option<i64>,
    pub last_aired: Option<i64>,
    pub status: Option<MetadataStatus>,
    pub tagline: Option<String>,
    pub next_aired: Option<i64>,
    pub genres: Vec<MetadataGenre>,
    pub content_ratings: Vec<ContentRating>,
    pub cast: Vec<CastCredit>,
    pub recommendations: Vec<Recommendation>,
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
    pub first_aired: Option<i64>,
    pub last_aired: Option<i64>,
    pub status: Option<MetadataStatus>,
    pub tagline: Option<String>,
    pub genres: Vec<MetadataGenre>,
    pub content_ratings: Vec<ContentRating>,
    pub cast: Vec<CastCredit>,
    pub recommendations: Vec<Recommendation>,
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
    pub first_aired: Option<i64>,
    pub last_aired: Option<i64>,
    pub status: Option<MetadataStatus>,
    pub tagline: Option<String>,
    pub next_aired: Option<i64>,
    pub genres: Vec<MetadataGenre>,
    pub content_ratings: Vec<ContentRating>,
    pub recommendations: Vec<Recommendation>,
    pub images: ImageSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMetadata {
    pub item_id: String,
    pub name: String,
    pub description: Option<String>,
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub first_aired: Option<i64>,
    pub last_aired: Option<i64>,
    pub status: Option<MetadataStatus>,
    pub tagline: Option<String>,
    pub next_aired: Option<i64>,
    pub genres: Vec<MetadataGenre>,
    pub content_ratings: Vec<ContentRating>,
    pub recommendations: Vec<Recommendation>,
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

    async fn lookup_people_metadata(
        &self,
        provider_person_ids: &[String],
    ) -> Result<Vec<PersonMetadata>>;

    async fn match_movie_root(
        &self,
        req: MovieRootMatchRequest,
    ) -> Result<Vec<Scored<MovieCandidate>>>;

    async fn lookup_movie_metadata(&self, candidate: &MovieCandidate) -> Result<MovieMetadata>;
}
