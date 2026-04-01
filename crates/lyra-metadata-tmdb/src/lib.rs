use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use lyra_metadata::{
    EpisodeMetadata, ImageSet, MetadataProvider, MovieCandidate, MovieMetadata,
    MovieRootMatchRequest, RootMatchHint, Scored, SeasonMetadata, SeriesCandidate, SeriesItem,
    SeriesItemsRequest, SeriesItemsResult, SeriesMetadata, SeriesRootMatchRequest,
};
use ratelimit::Ratelimiter;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tokio::time::sleep;

const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";
const TMDB_API_KEY: &str = "f81a38fe9eba82e5dc3695a7406068bd";
const CACHE_TTL: Duration = Duration::from_hours(24);

#[derive(Clone)]
pub struct TmdbMetadataProvider {
    client: Client,
    ratelimiter: Arc<Ratelimiter>,
    cache: Arc<Mutex<HashMap<String, CachedResponse>>>,
}

#[derive(Clone)]
struct CachedResponse {
    value: serde_json::Value,
    expires_at: Instant,
}

impl Default for TmdbMetadataProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TmdbMetadataProvider {
    pub fn new() -> Self {
        let ratelimiter = Ratelimiter::builder(1, Duration::from_secs(1))
            .max_tokens(5)
            .initial_available(1)
            .build()
            .expect("invalid TMDb rate limiter config");

        Self {
            client: Client::new(),
            ratelimiter: Arc::new(ratelimiter),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn rate_limit_wait(&self) {
        loop {
            match self.ratelimiter.try_wait() {
                Ok(()) => return,
                Err(wait_for) => sleep(wait_for).await,
            }
        }
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T> {
        let cache_key = cache_key(path, query);
        if let Some(cached_value) = self.get_cached_json(&cache_key).await {
            return serde_json::from_value(cached_value)
                .context("failed to decode cached TMDb data");
        }

        self.rate_limit_wait().await;
        let mut params = query.to_vec();
        params.push(("api_key", TMDB_API_KEY.to_string()));

        let url = format!("{TMDB_API_BASE}{path}");
        let value = self
            .client
            .get(url)
            .query(&params)
            .send()
            .await
            .context("failed to send TMDb request")?
            .error_for_status()
            .context("TMDb returned an error response")?
            .json::<serde_json::Value>()
            .await
            .context("failed to decode TMDb response body")?;

        {
            let mut cache = self.cache.lock().await;
            cache.insert(
                cache_key,
                CachedResponse {
                    value: value.clone(),
                    expires_at: Instant::now() + CACHE_TTL,
                },
            );
        }

        serde_json::from_value(value).context("failed to decode TMDb response into target type")
    }

    async fn get_cached_json(&self, cache_key: &str) -> Option<serde_json::Value> {
        let mut cache = self.cache.lock().await;
        let now = Instant::now();
        cache.retain(|_, entry| entry.expires_at > now);
        cache.get(cache_key).map(|entry| entry.value.clone())
    }

    async fn get_imdb_id_for_tv(&self, tmdb_id: u64) -> Result<Option<String>> {
        let external_ids: ExternalIds = self
            .get_json(&format!("/tv/{tmdb_id}/external_ids"), &[])
            .await?;
        Ok(empty_to_none(external_ids.imdb_id))
    }

    async fn get_imdb_id_for_movie(&self, tmdb_id: u64) -> Result<Option<String>> {
        let external_ids: ExternalIds = self
            .get_json(&format!("/movie/{tmdb_id}/external_ids"), &[])
            .await?;
        Ok(empty_to_none(external_ids.imdb_id))
    }
}

#[async_trait]
impl MetadataProvider for TmdbMetadataProvider {
    fn id(&self) -> &'static str {
        "tmdb"
    }

    async fn match_series_root(
        &self,
        req: SeriesRootMatchRequest,
    ) -> Result<Vec<Scored<SeriesCandidate>>> {
        let mut candidates = Vec::new();
        if let Some(tmdb_id) = req.hint.tmdb_id {
            let details: TvDetails = self.get_json(&format!("/tv/{tmdb_id}"), &[]).await?;
            candidates.push(Scored {
                value: SeriesCandidate {
                    tmdb_id: details.id,
                    name: details.name,
                    first_air_year: parse_year(details.first_air_date.as_deref()),
                },
                score: 1.0,
            });
            return Ok(candidates);
        }

        if let Some(imdb_id) = req.hint.imdb_id.as_deref() {
            let found: FindResponse = self
                .get_json(
                    &format!("/find/{imdb_id}"),
                    &[("external_source", "imdb_id".to_string())],
                )
                .await?;
            if let Some(first) = found.tv_results.first() {
                candidates.push(Scored {
                    value: SeriesCandidate {
                        tmdb_id: first.id,
                        name: first.name.clone(),
                        first_air_year: parse_year(first.first_air_date.as_deref()),
                    },
                    score: 0.98,
                });
                return Ok(candidates);
            }
        }

        let search: SearchResponse<TvSearchResult> = self
            .get_json(
                "/search/tv",
                &search_query_params(&req.hint, "first_air_date_year"),
            )
            .await?;
        Ok(score_series_candidates(&req.hint, search.results))
    }

    async fn lookup_series_metadata(&self, candidate: &SeriesCandidate) -> Result<SeriesMetadata> {
        let details: TvDetails = self
            .get_json(&format!("/tv/{}", candidate.tmdb_id), &[])
            .await?;
        let imdb_id = self.get_imdb_id_for_tv(details.id).await?;

        Ok(SeriesMetadata {
            imdb_id,
            tmdb_id: Some(details.id),
            name: details.name,
            description: empty_to_none(details.overview),
            score_display: score_display(details.vote_average),
            score_normalized: score_normalized(details.vote_average),
            first_aired: parse_date(details.first_air_date.as_deref()),
            last_aired: parse_date(details.last_air_date.as_deref()),
            images: ImageSet {
                poster_url: image_url(details.poster_path.as_deref(), "w780"),
                thumbnail_url: image_url(details.poster_path.as_deref(), "w342"),
                background_url: image_url(details.backdrop_path.as_deref(), "w1280"),
            },
        })
    }

    async fn lookup_series_items(&self, req: SeriesItemsRequest) -> Result<SeriesItemsResult> {
        let season_numbers = req
            .items
            .iter()
            .filter_map(|item| item.season_number)
            .collect::<HashSet<_>>();

        let mut season_rows = Vec::new();
        let mut episode_rows = Vec::new();
        for season_number in season_numbers {
            let season_details: TvSeasonDetails = self
                .get_json(
                    &format!("/tv/{}/season/{season_number}", req.candidate.tmdb_id),
                    &[],
                )
                .await?;

            season_rows.push(SeasonMetadata {
                root_id: req.root_id.clone(),
                season_number,
                name: season_details.name,
                description: empty_to_none(season_details.overview),
                score_display: None,
                score_normalized: None,
                first_aired: parse_date(season_details.air_date.as_deref()),
                last_aired: parse_date(season_details.air_date.as_deref()),
                images: ImageSet {
                    poster_url: image_url(season_details.poster_path.as_deref(), "w780"),
                    thumbnail_url: image_url(season_details.poster_path.as_deref(), "w342"),
                    background_url: None,
                },
            });

            let episodes_by_number = season_details
                .episodes
                .into_iter()
                .map(|episode| (episode.episode_number, episode))
                .collect::<HashMap<_, _>>();

            for item in req
                .items
                .iter()
                .filter(|item| item.season_number == Some(season_number))
            {
                let Some(episode_number) = item.episode_number else {
                    continue;
                };
                let Some(tmdb_episode) = episodes_by_number.get(&episode_number) else {
                    continue;
                };
                episode_rows.push(episode_metadata_from_item(item, tmdb_episode));
            }
        }

        Ok(SeriesItemsResult {
            seasons: season_rows,
            episodes: episode_rows,
        })
    }

    async fn match_movie_root(
        &self,
        req: MovieRootMatchRequest,
    ) -> Result<Vec<Scored<MovieCandidate>>> {
        let mut candidates = Vec::new();
        if let Some(tmdb_id) = req.hint.tmdb_id {
            let details: MovieDetails = self.get_json(&format!("/movie/{tmdb_id}"), &[]).await?;
            candidates.push(Scored {
                value: MovieCandidate {
                    tmdb_id: details.id,
                    name: details.title,
                    release_year: parse_year(details.release_date.as_deref()),
                },
                score: 1.0,
            });
            return Ok(candidates);
        }

        if let Some(imdb_id) = req.hint.imdb_id.as_deref() {
            let found: FindResponse = self
                .get_json(
                    &format!("/find/{imdb_id}"),
                    &[("external_source", "imdb_id".to_string())],
                )
                .await?;
            if let Some(first) = found.movie_results.first() {
                candidates.push(Scored {
                    value: MovieCandidate {
                        tmdb_id: first.id,
                        name: first.title.clone(),
                        release_year: parse_year(first.release_date.as_deref()),
                    },
                    score: 0.98,
                });
                return Ok(candidates);
            }
        }

        let search: SearchResponse<MovieSearchResult> = self
            .get_json("/search/movie", &search_query_params(&req.hint, "year"))
            .await?;
        Ok(score_movie_candidates(&req.hint, search.results))
    }

    async fn lookup_movie_metadata(&self, candidate: &MovieCandidate) -> Result<MovieMetadata> {
        let details: MovieDetails = self
            .get_json(&format!("/movie/{}", candidate.tmdb_id), &[])
            .await?;
        let imdb_id = self.get_imdb_id_for_movie(details.id).await?;

        Ok(MovieMetadata {
            imdb_id,
            tmdb_id: Some(details.id),
            name: details.title,
            description: empty_to_none(details.overview),
            score_display: score_display(details.vote_average),
            score_normalized: score_normalized(details.vote_average),
            first_aired: parse_date(details.release_date.as_deref()),
            last_aired: parse_date(details.release_date.as_deref()),
            images: ImageSet {
                poster_url: image_url(details.poster_path.as_deref(), "w780"),
                thumbnail_url: image_url(details.poster_path.as_deref(), "w342"),
                background_url: image_url(details.backdrop_path.as_deref(), "w1280"),
            },
        })
    }
}

fn episode_metadata_from_item(item: &SeriesItem, episode: &TvEpisodeDetails) -> EpisodeMetadata {
    EpisodeMetadata {
        item_id: item.item_id.clone(),
        name: empty_to_none(episode.name.clone()).unwrap_or_else(|| item.name.clone()),
        description: empty_to_none(episode.overview.clone()),
        score_display: score_display(episode.vote_average),
        score_normalized: score_normalized(episode.vote_average),
        first_aired: parse_date(episode.air_date.as_deref()),
        last_aired: parse_date(episode.air_date.as_deref()),
        images: ImageSet {
            poster_url: image_url(episode.still_path.as_deref(), "w780"),
            thumbnail_url: image_url(episode.still_path.as_deref(), "w300"),
            background_url: image_url(episode.still_path.as_deref(), "w780"),
        },
    }
}

fn search_query_params(
    hint: &RootMatchHint,
    year_key: &'static str,
) -> Vec<(&'static str, String)> {
    let mut params = vec![
        ("query", hint.title.clone()),
        ("include_adult", "false".to_string()),
    ];
    if let Some(year) = hint.start_year {
        params.push((year_key, year.to_string()));
    }
    params
}

fn score_series_candidates(
    hint: &RootMatchHint,
    rows: Vec<TvSearchResult>,
) -> Vec<Scored<SeriesCandidate>> {
    score_candidates(
        hint,
        rows.into_iter().map(|row| {
            (
                row.id,
                row.name,
                parse_year(row.first_air_date.as_deref()),
                row.first_air_date,
            )
        }),
        |tmdb_id, name, year| SeriesCandidate {
            tmdb_id,
            name,
            first_air_year: year,
        },
    )
}

fn score_movie_candidates(
    hint: &RootMatchHint,
    rows: Vec<MovieSearchResult>,
) -> Vec<Scored<MovieCandidate>> {
    score_candidates(
        hint,
        rows.into_iter().map(|row| {
            (
                row.id,
                row.title,
                parse_year(row.release_date.as_deref()),
                row.release_date,
            )
        }),
        |tmdb_id, name, year| MovieCandidate {
            tmdb_id,
            name,
            release_year: year,
        },
    )
}

fn score_candidates<T, I, F>(hint: &RootMatchHint, rows: I, map_fn: F) -> Vec<Scored<T>>
where
    I: IntoIterator<Item = (u64, String, Option<i32>, Option<String>)>,
    F: Fn(u64, String, Option<i32>) -> T,
{
    let expected = normalize_title(&hint.title);
    let mut scored = rows
        .into_iter()
        .map(|(tmdb_id, name, year, _date)| {
            let actual = normalize_title(&name);
            let mut score = if expected == actual {
                0.92
            } else if actual.contains(&expected) || expected.contains(&actual) {
                0.80
            } else {
                0.55
            };
            if hint.start_year.is_some() && hint.start_year == year {
                score += 0.08;
            }
            Scored {
                value: map_fn(tmdb_id, name, year),
                score,
            }
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| b.score.total_cmp(&a.score));
    scored.truncate(10);
    scored
}

fn normalize_title(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn cache_key(path: &str, query: &[(&str, String)]) -> String {
    let mut parts = query
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    parts.sort_unstable();
    format!("{path}?{}", parts.join("&"))
}

fn parse_date(value: Option<&str>) -> Option<i64> {
    let value = value?;
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?;
    date.and_hms_opt(0, 0, 0).map(|ts| ts.and_utc().timestamp())
}

fn parse_year(value: Option<&str>) -> Option<i32> {
    value.and_then(|raw| raw.split('-').next())?.parse().ok()
}

fn image_url(path: Option<&str>, size: &str) -> Option<String> {
    let path = path?;
    Some(format!("{TMDB_IMAGE_BASE}/{size}{path}"))
}

fn score_display(vote: Option<f64>) -> Option<String> {
    vote.map(|score| format!("{score:.1}/10"))
}

fn score_normalized(vote: Option<f64>) -> Option<i64> {
    vote.map(|score| (score * 10.0).round() as i64)
}

fn empty_to_none(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then_some(trimmed.to_string())
    })
}

#[derive(Debug, Deserialize)]
struct SearchResponse<T> {
    results: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct FindResponse {
    #[serde(default)]
    tv_results: Vec<TvSearchResult>,
    #[serde(default)]
    movie_results: Vec<MovieSearchResult>,
}

#[derive(Debug, Deserialize)]
struct ExternalIds {
    imdb_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TvSearchResult {
    id: u64,
    #[serde(default)]
    name: String,
    first_air_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MovieSearchResult {
    id: u64,
    #[serde(default)]
    title: String,
    release_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TvDetails {
    id: u64,
    #[serde(default)]
    name: String,
    overview: Option<String>,
    vote_average: Option<f64>,
    first_air_date: Option<String>,
    last_air_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MovieDetails {
    id: u64,
    #[serde(default)]
    title: String,
    overview: Option<String>,
    vote_average: Option<f64>,
    release_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TvSeasonDetails {
    #[serde(default)]
    name: String,
    overview: Option<String>,
    air_date: Option<String>,
    poster_path: Option<String>,
    #[serde(default)]
    episodes: Vec<TvEpisodeDetails>,
}

#[derive(Debug, Deserialize)]
struct TvEpisodeDetails {
    episode_number: i32,
    name: Option<String>,
    overview: Option<String>,
    vote_average: Option<f64>,
    air_date: Option<String>,
    still_path: Option<String>,
}
