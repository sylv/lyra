use crate::config::get_config;
use anyhow::Result;
use lru::LruCache;
use ratelimit::Ratelimiter;
use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

pub const TMDB_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/original";
const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct TMDBClient {
    client: Client,
    api_key: String,
    cache: Arc<Mutex<LruCache<String, String>>>,
    limiter: Arc<Ratelimiter>,
}

impl TMDBClient {
    pub fn new() -> Self {
        let limiter = Ratelimiter::builder(5, Duration::from_secs(1))
            .max_tokens(20)
            .build()
            .unwrap();

        Self {
            client: Client::builder()
                .user_agent(APP_USER_AGENT)
                .build()
                .unwrap(),
            api_key: get_config().tmdb_api_key.clone(),
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))),
            limiter: Arc::new(limiter),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str, params: &[(&str, &str)]) -> Result<T> {
        let url_string = format!("{}{}", TMDB_BASE_URL, path);
        let mut url = reqwest::Url::parse(&url_string)?;
        url.query_pairs_mut().append_pair("api_key", &self.api_key);
        for (key, value) in params {
            url.query_pairs_mut().append_pair(key, value);
        }

        let cache_key = url.to_string();

        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached_response) = cache.get(&cache_key) {
                tracing::info!("cache hit for {}", cache_key);
                return Ok(serde_json::from_str(cached_response)?);
            }
        }

        self.wait_for_rate_limit().await;

        tracing::info!("cache miss for {}", cache_key);
        let res = self.client.get(url).send().await?.error_for_status()?;
        let json_text = res.text().await?;

        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(cache_key, json_text.clone());
        }

        Ok(serde_json::from_str(&json_text)?)
    }

    async fn wait_for_rate_limit(&self) {
        loop {
            if self.limiter.try_wait().is_ok() {
                break;
            }

            sleep(Duration::from_millis(50)).await;
        }
    }

    pub async fn search_movie(
        &self,
        query: &str,
        year: Option<&str>,
    ) -> Result<SearchResponse<MovieSearchResult>> {
        let mut params = vec![("query", query)];
        if let Some(year) = year {
            params.push(("year", year));
        }
        self.get("/search/movie", &params).await
    }

    pub async fn search_tv(
        &self,
        query: &str,
        year: Option<&str>,
    ) -> Result<SearchResponse<TvSearchResult>> {
        let mut params = vec![("query", query)];
        if let Some(year) = year {
            params.push(("first_air_date_year", year));
        }
        self.get("/search/tv", &params).await
    }

    pub async fn get_movie_details(&self, movie_id: i64) -> Result<MovieDetails> {
        self.get(&format!("/movie/{}", movie_id), &[]).await
    }

    pub async fn get_tv_show_details(&self, tv_id: i64) -> Result<TvShowDetails> {
        self.get(&format!("/tv/{}", tv_id), &[]).await
    }

    pub async fn get_tv_season_details(
        &self,
        tv_id: i64,
        season_number: i64,
    ) -> Result<TvSeasonDetails> {
        self.get(&format!("/tv/{}/season/{}", tv_id, season_number), &[])
            .await
    }
}

#[derive(Deserialize, Debug)]
pub struct SearchResponse<T> {
    pub results: Vec<T>,
}

#[derive(Deserialize, Debug)]
pub struct MovieSearchResult {
    pub id: i64,
    pub title: String,
    pub release_date: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TvSearchResult {
    pub id: i64,
    pub name: String,
    pub first_air_date: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MovieDetails {
    pub id: i64,
    pub title: String,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<i64>,
    pub vote_average: Option<f64>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TvShowDetails {
    pub id: i64,
    pub name: String,
    pub overview: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: Option<f64>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub seasons: Vec<TvSeason>,
}

#[derive(Deserialize, Debug)]
pub struct TvSeason {
    pub id: i64,
    pub season_number: i64,
}

#[derive(Deserialize, Debug)]
pub struct TvSeasonDetails {
    pub id: i64,
    pub name: String,
    pub overview: Option<String>,
    pub season_number: i64,
    pub episodes: Vec<TvEpisode>,
}

#[derive(Deserialize, Debug)]
pub struct TvEpisode {
    pub id: i64,
    pub name: String,
    pub overview: Option<String>,
    pub episode_number: i64,
    pub runtime: Option<i64>,
    pub vote_average: Option<f64>,
    pub still_path: Option<String>,
}
