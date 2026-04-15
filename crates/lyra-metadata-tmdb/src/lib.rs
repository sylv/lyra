use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use lyra_metadata::{
    CastCredit, ContentRating, EpisodeMetadata, ImageSet, MetadataGenre, MetadataImage,
    MetadataImageKind, MetadataProvider, MetadataStatus, MovieCandidate, MovieMetadata,
    MovieRootMatchRequest, PersonMetadata, Recommendation, RecommendedMediaKind, RootMatchHint,
    Scored, SeasonMetadata, SeriesCandidate, SeriesItem, SeriesItemsRequest, SeriesItemsResult,
    SeriesMetadata, SeriesRootMatchRequest,
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
            let details: TvSearchResult = self.get_json(&format!("/tv/{tmdb_id}"), &[]).await?;
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
            .get_json(
                &format!("/tv/{}", candidate.tmdb_id),
                &[(
                    "append_to_response",
                    "external_ids,content_ratings,aggregate_credits,recommendations,images"
                        .to_string(),
                )],
            )
            .await?;

        Ok(SeriesMetadata {
            imdb_id: empty_to_none(details.external_ids.and_then(|ids| ids.imdb_id)),
            tmdb_id: Some(details.id),
            name: details.name,
            description: empty_to_none(details.overview),
            score_display: score_display(details.vote_average),
            score_normalized: score_normalized(details.vote_average),
            first_aired: parse_date(details.first_air_date.as_deref()),
            last_aired: parse_date(details.last_air_date.as_deref()),
            status: map_tv_status(details.status.as_deref()),
            tagline: empty_to_none(details.tagline),
            next_aired: details
                .next_episode_to_air
                .and_then(|episode| parse_date(episode.air_date.as_deref())),
            genres: map_genres(self.id(), details.genres),
            content_ratings: details
                .content_ratings
                .map(|ratings| map_tv_content_ratings(ratings.results))
                .unwrap_or_default(),
            cast: details
                .aggregate_credits
                .map(|credits| map_cast(credits.cast))
                .unwrap_or_default(),
            recommendations: details
                .recommendations
                .map(|rows| map_tv_recommendations(rows.results))
                .unwrap_or_default(),
            images: images_from_details(
                details.poster_path.as_deref(),
                details.backdrop_path.as_deref(),
                details.images,
            ),
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
                status: None,
                tagline: None,
                next_aired: None,
                genres: Vec::new(),
                content_ratings: Vec::new(),
                recommendations: Vec::new(),
                images: ImageSet {
                    posters: collect_single_image(
                        MetadataImageKind::Poster,
                        season_details.poster_path.as_deref(),
                        "w780",
                    ),
                    thumbnails: collect_single_image(
                        MetadataImageKind::Thumbnail,
                        season_details.poster_path.as_deref(),
                        "w342",
                    ),
                    backdrops: Vec::new(),
                    logos: Vec::new(),
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
            let details: MovieSearchResult =
                self.get_json(&format!("/movie/{tmdb_id}"), &[]).await?;
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
            .get_json(
                &format!("/movie/{}", candidate.tmdb_id),
                &[(
                    "append_to_response",
                    "external_ids,release_dates,credits,recommendations,images".to_string(),
                )],
            )
            .await?;

        Ok(MovieMetadata {
            imdb_id: empty_to_none(details.external_ids.and_then(|ids| ids.imdb_id)),
            tmdb_id: Some(details.id),
            name: details.title,
            description: empty_to_none(details.overview),
            score_display: score_display(details.vote_average),
            score_normalized: score_normalized(details.vote_average),
            first_aired: parse_date(details.release_date.as_deref()),
            last_aired: parse_date(details.release_date.as_deref()),
            status: map_movie_status(details.status.as_deref()),
            tagline: empty_to_none(details.tagline),
            genres: map_genres(self.id(), details.genres),
            content_ratings: details
                .release_dates
                .map(|dates| map_movie_content_ratings(dates.results))
                .unwrap_or_default(),
            cast: details
                .credits
                .map(|credits| map_cast(credits.cast))
                .unwrap_or_default(),
            recommendations: details
                .recommendations
                .map(|rows| map_movie_recommendations(rows.results))
                .unwrap_or_default(),
            images: images_from_details(
                details.poster_path.as_deref(),
                details.backdrop_path.as_deref(),
                details.images,
            ),
        })
    }

    async fn lookup_people_metadata(
        &self,
        provider_person_ids: &[String],
    ) -> Result<Vec<PersonMetadata>> {
        let mut people = Vec::new();

        for provider_person_id in provider_person_ids {
            let Ok(tmdb_person_id) = provider_person_id.parse::<u64>() else {
                continue;
            };
            let details: TmdbPersonDetails = self
                .get_json(&format!("/person/{tmdb_person_id}"), &[])
                .await?;
            people.push(map_person_metadata(details));
        }

        Ok(people)
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
        status: None,
        tagline: None,
        next_aired: None,
        genres: Vec::new(),
        content_ratings: Vec::new(),
        recommendations: Vec::new(),
        images: ImageSet {
            posters: Vec::new(),
            thumbnails: collect_single_image(
                MetadataImageKind::Thumbnail,
                episode.still_path.as_deref(),
                "w300",
            ),
            backdrops: Vec::new(),
            logos: Vec::new(),
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

fn map_tv_status(status: Option<&str>) -> Option<MetadataStatus> {
    match status? {
        "Returning Series" => Some(MetadataStatus::Returning),
        "Ended" => Some(MetadataStatus::Finished),
        "Canceled" | "Cancelled" => Some(MetadataStatus::Cancelled),
        "In Production" => Some(MetadataStatus::Returning),
        "Planned" | "Pilot" => Some(MetadataStatus::Upcoming),
        _ => None,
    }
}

fn map_movie_status(status: Option<&str>) -> Option<MetadataStatus> {
    match status? {
        "Released" => Some(MetadataStatus::Released),
        "Canceled" | "Cancelled" => Some(MetadataStatus::Cancelled),
        "Rumored" | "Planned" | "In Production" | "Post Production" => {
            Some(MetadataStatus::Upcoming)
        }
        _ => None,
    }
}

fn map_genres(provider_id: &str, genres: Vec<TmdbGenre>) -> Vec<MetadataGenre> {
    genres
        .into_iter()
        .map(|genre| MetadataGenre {
            provider_id: provider_id.to_string(),
            external_id: Some(genre.id.to_string()),
            name: genre.name,
        })
        .collect()
}

fn map_tv_content_ratings(rows: Vec<TvContentRating>) -> Vec<ContentRating> {
    rows.into_iter()
        .filter_map(|row| {
            let rating = empty_to_none(row.rating)?;
            Some(ContentRating {
                country_code: row.iso_3166_1,
                rating,
                release_date: None,
                release_type: None,
            })
        })
        .collect()
}

fn map_movie_content_ratings(rows: Vec<MovieReleaseDatesCountry>) -> Vec<ContentRating> {
    let mut ratings = Vec::new();
    for row in rows {
        for release in row.release_dates {
            let Some(rating) = empty_to_none(release.certification) else {
                continue;
            };
            ratings.push(ContentRating {
                country_code: row.iso_3166_1.clone(),
                rating,
                release_date: parse_date(release.release_date.as_deref()),
                release_type: release.release_type,
            });
        }
    }
    ratings
}

fn map_cast<T: CreditLike>(rows: Vec<T>) -> Vec<CastCredit> {
    rows.into_iter()
        .take(20)
        .map(|row| CastCredit {
            provider_person_id: row.id().to_string(),
            name: row.name().to_string(),
            character_name: empty_to_none(Some(row.character().to_string())),
            department: None,
        })
        .collect()
}

fn map_person_metadata(person: TmdbPersonDetails) -> PersonMetadata {
    PersonMetadata {
        provider_person_id: person.id.to_string(),
        name: person.name,
        birthday: parse_date_str(person.birthday.as_deref()),
        description: empty_to_none(person.biography),
        profile_image_url: image_url(person.profile_path.as_deref(), "w342"),
    }
}

fn map_tv_recommendations(rows: Vec<TvSearchResult>) -> Vec<Recommendation> {
    rows.into_iter()
        .map(|row| Recommendation {
            media_kind: RecommendedMediaKind::Series,
            tmdb_id: Some(row.id),
            imdb_id: None,
            name: row.name,
            first_aired: parse_date(row.first_air_date.as_deref()),
        })
        .collect()
}

fn map_movie_recommendations(rows: Vec<MovieSearchResult>) -> Vec<Recommendation> {
    rows.into_iter()
        .map(|row| Recommendation {
            media_kind: RecommendedMediaKind::Movie,
            tmdb_id: Some(row.id),
            imdb_id: None,
            name: row.title,
            first_aired: parse_date(row.release_date.as_deref()),
        })
        .collect()
}

fn images_from_details(
    primary_poster_path: Option<&str>,
    primary_backdrop_path: Option<&str>,
    images: Option<TmdbImages>,
) -> ImageSet {
    let mut posters = collect_single_image(MetadataImageKind::Poster, primary_poster_path, "w780");
    let mut thumbnails =
        collect_single_image(MetadataImageKind::Thumbnail, primary_poster_path, "w342");
    let mut backdrops =
        collect_single_image(MetadataImageKind::Backdrop, primary_backdrop_path, "w1280");
    let mut logos = Vec::new();

    if let Some(images) = images {
        extend_unique_images(
            &mut posters,
            images.posters.clone(),
            MetadataImageKind::Poster,
            "w780",
        );
        extend_unique_images(
            &mut thumbnails,
            images.posters,
            MetadataImageKind::Thumbnail,
            "w342",
        );
        extend_unique_images(
            &mut backdrops,
            images.backdrops,
            MetadataImageKind::Backdrop,
            "w1280",
        );

        let mut logo_rows = images.logos;
        logo_rows.sort_by_key(|row| logo_sort_key(row));
        extend_unique_images(&mut logos, logo_rows, MetadataImageKind::Logo, "original");
    }

    ImageSet {
        posters,
        thumbnails,
        backdrops,
        logos,
    }
}

fn logo_sort_key(row: &TmdbImage) -> (u8, i64, i64, u8) {
    let language_rank = match row.iso_639_1.as_deref() {
        Some("en") => 0,
        Some(_) => 1,
        None => 2,
    };
    let vote_average_rank = -(row.vote_average * 1000.0).round() as i64;
    let vote_count_rank = -row.vote_count;
    let svg_rank = if row.file_type.as_deref() == Some(".svg") {
        0
    } else {
        1
    };

    (language_rank, vote_average_rank, vote_count_rank, svg_rank)
}

fn collect_single_image(
    kind: MetadataImageKind,
    path: Option<&str>,
    size: &str,
) -> Vec<MetadataImage> {
    image_url(path, size)
        .map(|url| {
            vec![MetadataImage {
                kind,
                url,
                language: None,
                vote_average: None,
                vote_count: None,
                width: None,
                height: None,
                file_type: None,
            }]
        })
        .unwrap_or_default()
}

fn extend_unique_images(
    target: &mut Vec<MetadataImage>,
    rows: Vec<TmdbImage>,
    kind: MetadataImageKind,
    size: &str,
) {
    let mut seen = target
        .iter()
        .map(|image| image.url.clone())
        .collect::<HashSet<_>>();
    for row in rows {
        let Some(url) = image_url(row.file_path.as_deref(), size) else {
            continue;
        };
        if !seen.insert(url.clone()) {
            continue;
        }
        target.push(MetadataImage {
            kind,
            url,
            language: empty_to_none(row.iso_639_1),
            vote_average: Some(row.vote_average),
            vote_count: Some(row.vote_count),
            width: row.width,
            height: row.height,
            file_type: empty_to_none(row.file_type),
        });
    }
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
    let trimmed = value.get(..10).unwrap_or(value);
    let date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()?;
    date.and_hms_opt(0, 0, 0).map(|ts| ts.and_utc().timestamp())
}

fn parse_date_str(value: Option<&str>) -> Option<String> {
    let value = value?;
    let trimmed = value.get(..10).unwrap_or(value);
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .ok()
        .map(|d| d.to_string())
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

trait CreditLike {
    fn id(&self) -> u64;
    fn name(&self) -> &str;
    fn character(&self) -> &str;
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
    status: Option<String>,
    tagline: Option<String>,
    #[serde(default)]
    genres: Vec<TmdbGenre>,
    external_ids: Option<ExternalIds>,
    content_ratings: Option<TvContentRatingsResponse>,
    aggregate_credits: Option<TvAggregateCredits>,
    recommendations: Option<SearchResponse<TvSearchResult>>,
    images: Option<TmdbImages>,
    next_episode_to_air: Option<NextEpisode>,
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
    status: Option<String>,
    tagline: Option<String>,
    #[serde(default)]
    genres: Vec<TmdbGenre>,
    external_ids: Option<ExternalIds>,
    release_dates: Option<MovieReleaseDatesResponse>,
    credits: Option<MovieCredits>,
    recommendations: Option<SearchResponse<MovieSearchResult>>,
    images: Option<TmdbImages>,
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

#[derive(Debug, Deserialize)]
struct TmdbGenre {
    id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct TvContentRatingsResponse {
    #[serde(default)]
    results: Vec<TvContentRating>,
}

#[derive(Debug, Deserialize)]
struct TvContentRating {
    iso_3166_1: String,
    rating: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MovieReleaseDatesResponse {
    #[serde(default)]
    results: Vec<MovieReleaseDatesCountry>,
}

#[derive(Debug, Deserialize)]
struct MovieReleaseDatesCountry {
    iso_3166_1: String,
    #[serde(default)]
    release_dates: Vec<MovieReleaseDate>,
}

#[derive(Debug, Deserialize)]
struct MovieReleaseDate {
    certification: Option<String>,
    release_date: Option<String>,
    release_type: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MovieCredits {
    #[serde(default)]
    cast: Vec<TmdbCast>,
}

#[derive(Debug, Deserialize)]
struct TvAggregateCredits {
    #[serde(default)]
    cast: Vec<TmdbAggregateCast>,
}

#[derive(Debug, Deserialize)]
struct TmdbCast {
    id: u64,
    name: String,
    character: Option<String>,
}

impl CreditLike for TmdbCast {
    fn id(&self) -> u64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn character(&self) -> &str {
        self.character.as_deref().unwrap_or("")
    }
}

#[derive(Debug, Deserialize)]
struct TmdbAggregateCast {
    id: u64,
    name: String,
    #[serde(default)]
    roles: Vec<TmdbAggregateRole>,
}

impl CreditLike for TmdbAggregateCast {
    fn id(&self) -> u64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn character(&self) -> &str {
        self.roles
            .first()
            .and_then(|role| role.character.as_deref())
            .unwrap_or("")
    }
}

#[derive(Debug, Deserialize)]
struct TmdbAggregateRole {
    character: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TmdbImages {
    #[serde(default)]
    backdrops: Vec<TmdbImage>,
    #[serde(default)]
    logos: Vec<TmdbImage>,
    #[serde(default)]
    posters: Vec<TmdbImage>,
}

#[derive(Debug, Deserialize, Clone)]
struct TmdbImage {
    file_path: Option<String>,
    iso_639_1: Option<String>,
    vote_average: f64,
    vote_count: i64,
    width: Option<i64>,
    height: Option<i64>,
    file_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NextEpisode {
    air_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TmdbPersonDetails {
    id: u64,
    name: String,
    biography: Option<String>,
    birthday: Option<String>,
    profile_path: Option<String>,
}
