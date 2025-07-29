use crate::{
    matcher::parser::{get_imdb_id, get_tmdb_id, parse_file_name},
    tmdb::{MovieDetails, TMDBClient, TvShowDetails},
};
use anyhow::Result;
use chrono::Datelike;

pub enum MatchResult {
    Movie(MovieDetails),
    Series {
        show: TvShowDetails,
        parsed: ParsedFile,
    },
}

#[derive(Debug)]
pub struct ParsedFile {
    pub title: String,
    pub year: Option<i32>,
    pub season_number: Option<i32>,
    pub episodes: Vec<i32>,
}

impl ParsedFile {
    pub fn is_movie(&self) -> bool {
        self.season_number.is_none()
    }

    pub fn is_series(&self) -> bool {
        self.season_number.is_some()
    }
}

pub enum Candidate {
    Movie {
        tmdb_id: i64,
        title: String,
        release_year: Option<i32>,
    },
    Series {
        tmdb_id: i64,
        title: String,
        first_air_year: Option<i32>,
        last_air_year: Option<i32>,
        season_count: Option<i32>,
        episode_count: Option<i32>,
    },
}

impl Candidate {
    fn is_allowed_match(&self, parsed_file: &ParsedFile) -> bool {
        match self {
            Candidate::Movie { release_year, .. } => {
                if !parsed_file.is_movie() {
                    tracing::warn!(
                        "discarding movie candidate '{}' because it is not a movie",
                        self.title()
                    );
                    return false;
                }

                if let Some(parsed_year) = parsed_file.year {
                    let Some(release_year) = release_year else {
                        // without a release year, we assume it isn't a match because if the file has a year
                        // but tmdb doesn't, its probably an unreleased movie with the same name etc
                        return false;
                    };

                    let diff = (parsed_year - release_year).abs();
                    if diff > 1 {
                        tracing::warn!(
                            "discarding movie candidate '{}' because it is from {} but the release year is {}",
                            self.title(),
                            parsed_year,
                            release_year
                        );
                        return false;
                    }
                }

                true
            }
            Candidate::Series {
                first_air_year,
                last_air_year,
                season_count,
                episode_count,
                ..
            } => {
                if !parsed_file.is_series() {
                    tracing::warn!(
                        "discarding series candidate '{}' because it is not a series",
                        self.title()
                    );
                    return false;
                }

                // ensure the candidate has the right number of seasons
                if let Some(season_count) = season_count {
                    if let Some(parsed_season) = parsed_file.season_number {
                        if parsed_season > *season_count {
                            tracing::warn!(
                                "discarding series candidate '{}' because it has {} seasons but the parsed file has season {}",
                                self.title(),
                                season_count,
                                parsed_season
                            );

                            return false;
                        }
                    }
                }

                // ensure the candidate has the right number of episodes
                // i imagine this is mostly only relevant for eg one piece with absolute episode ordering
                // where eg "episode 1022" would come up and the candidate only has 22 episodes in total
                if let Some(episode_count) = episode_count {
                    let highest_episode_number = parsed_file.episodes.iter().max().unwrap_or(&0);
                    if highest_episode_number > episode_count || episode_count == &0 {
                        tracing::warn!(
                            "discarding series candidate '{}' because it has {} episodes but the parsed file has episode {}",
                            self.title(),
                            episode_count,
                            highest_episode_number
                        );

                        return false;
                    }
                }

                if let Some(parsed_year) = parsed_file.year {
                    if let Some(first_air_year) = first_air_year {
                        if parsed_year + 1 < *first_air_year {
                            // if the file is from before the show started, its probably wrong.
                            // +1 is for the same reason as the movie year check
                            tracing::warn!(
                                "discarding series candidate '{}' because it is from {} but the show started in {}",
                                self.title(),
                                parsed_year,
                                first_air_year
                            );

                            return false;
                        }
                    }

                    if let Some(last_air_year) = last_air_year {
                        if parsed_year - 1 > *last_air_year {
                            tracing::warn!(
                                "discarding series candidate '{}' because it is from {} but the show ended in {}",
                                self.title(),
                                parsed_year,
                                last_air_year
                            );

                            return false;
                        }
                    }
                }

                true
            }
        }
    }

    fn title(&self) -> &str {
        match self {
            Candidate::Movie { title, .. } => title,
            Candidate::Series { title, .. } => title,
        }
    }
}

// todo: testing
// 2012 (2009) has like 100 results with identical names and only differing release years.
// test for shows that share a name with other shows
// test for shows with lots of regional variants, eg "The Office (US)" vs "The Office (UK)"
// shows with regional variants but one significantly more popular than the other, eg "Top Gear (UK)", often lacking the UK suffix, vs "Top Gear (US)"
// todo: sort by name/year score, then popularity
pub async fn match_file_to_metadata(
    tmdb_client: &TMDBClient,
    file_name: &str,
) -> Result<Option<MatchResult>> {
    let parsed_file = parse_file_name(file_name)?;

    if let Some(tmdb_id) = get_tmdb_id(file_name) {
        tracing::debug!(parsed_file = ?parsed_file, "matching file based on tmdb id");
        if parsed_file.is_series() {
            let show = tmdb_client.get_tv_show_details(tmdb_id, true).await?;
            return Ok(Some(MatchResult::Series {
                show,
                parsed: parsed_file,
            }));
        } else {
            let movie = tmdb_client.get_movie_details(tmdb_id, true).await?;
            return Ok(Some(MatchResult::Movie(movie)));
        }
    }

    if let Some(imdb_id) = get_imdb_id(file_name) {
        tracing::debug!(parsed_file = ?parsed_file, "matching item based on imdb id");

        let find_result = tmdb_client.find_by_imdb_id(&imdb_id).await?;
        if let Some(movie) = find_result.movie_results.first() {
            let movie_details = tmdb_client.get_movie_details(movie.id, true).await?;
            return Ok(Some(MatchResult::Movie(movie_details)));
        }

        if let Some(show) = find_result.tv_results.first() {
            if parsed_file.is_series() {
                let show_details = tmdb_client.get_tv_show_details(show.id, true).await?;
                return Ok(Some(MatchResult::Series {
                    show: show_details,
                    parsed: parsed_file,
                }));
            } else {
                tracing::error!(
                    "resolved '{}' to tv show '{}' but expected movie",
                    parsed_file.title,
                    show.name
                );

                return Ok(None);
            }
        }
    }

    tracing::debug!(parsed_file = ?parsed_file, "matching item based on heuristics");

    // todo: ideally we would sort candidates by the likeliness of it matching (eg, name/year distance)
    // which is what the first implementation did, but it breaks down in a lot of scenarios
    // (for example, "shingeki no kyojin" would fail to match "attack on titan", we would have to explicitly
    // handle alternate names), so for now we just use popularity-based ordering and filter out obviously wrong entries.
    // this might mean eg, one piece live action would be matched to an anime episode, but it's better than nothing for now.
    let haystack = get_candidates(tmdb_client, &parsed_file).await?;

    let mut checked_count = 0;
    let max_checks = 4;

    for candidate in haystack {
        if !candidate.is_allowed_match(&parsed_file) {
            // because this is a fast check we don't consider it for checked_count,
            // which is mostly so we don't hit tmdb with 20 requests looking for the right match
            continue;
        }

        checked_count += 1;
        if checked_count > max_checks {
            tracing::info!(
                "giving up search after checking {} candidates without finding a match",
                max_checks
            );

            break;
        }

        match candidate {
            Candidate::Movie { tmdb_id, .. } => {
                let movie = tmdb_client.get_movie_details(tmdb_id, true).await?;
                return Ok(Some(MatchResult::Movie(movie)));
            }
            Candidate::Series { tmdb_id, .. } => {
                let show = tmdb_client.get_tv_show_details(tmdb_id, true).await?;

                // for shows, we cannot get all the details we need from the search results.
                // so we have to do the is_allowed_match check again after updating the candidate with the full details
                let first_air_year = show
                    .first_air_date
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                    .map(|d| d.year());

                let last_air_year = show
                    .last_air_date
                    .as_ref()
                    .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                    .map(|d| d.year());

                let full_candidate = Candidate::Series {
                    tmdb_id: show.id,
                    title: show.name.clone(),
                    first_air_year: first_air_year,
                    last_air_year: last_air_year,
                    episode_count: show.number_of_episodes,
                    season_count: show.number_of_seasons,
                };

                if !full_candidate.is_allowed_match(&parsed_file) {
                    continue;
                }

                return Ok(Some(MatchResult::Series {
                    show,
                    parsed: parsed_file,
                }));
            }
        }
    }

    Ok(None)
}

async fn get_candidates(
    tmdb_client: &TMDBClient,
    parsed_file: &ParsedFile,
) -> Result<Vec<Candidate>> {
    let mut haystack = Vec::new();
    if parsed_file.is_series() {
        let results = tmdb_client.search_tv(&parsed_file.title, None).await?;
        for result in results.results {
            let release_year = result
                .first_air_date
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                .map(|d| d.year());

            haystack.push(Candidate::Series {
                title: result.name,
                episode_count: None,
                season_count: None,
                first_air_year: release_year,
                last_air_year: None,
                tmdb_id: result.id,
            });
        }
    } else {
        let results = tmdb_client.search_movie(&parsed_file.title, None).await?;
        for result in results.results {
            let release_year = result
                .release_date
                .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                .map(|d| d.year());

            haystack.push(Candidate::Movie {
                title: result.title,
                release_year,
                tmdb_id: result.id,
            });
        }
    }

    Ok(haystack)
}
