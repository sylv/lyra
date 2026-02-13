use crate::model::run_batched_inference;
use crate::parser::edition::parse_edition;
use crate::parser::ids::parse_ids;
use crate::parser::title::parse_title;
use parser::season_episode::parse_season_episode;
use parser::year::parse_year;
use serde::{Deserialize, Serialize};

mod infer_numbers;
mod model;
mod parser;
mod pattern;
pub mod should_ignore_path;
mod util;

#[derive(Debug, Clone)]
pub struct ParserContext {
    matched_ranges: Vec<std::ops::Range<usize>>,
}

impl ParserContext {
    pub fn new() -> Self {
        Self {
            matched_ranges: Vec::new(),
        }
    }

    pub fn add_match(&mut self, range: std::ops::Range<usize>) {
        self.matched_ranges.push(range);
    }

    pub fn overlaps_any(&self, range: &std::ops::Range<usize>) -> bool {
        self.matched_ranges
            .iter()
            .any(|existing| range.start < existing.end && existing.start < range.end)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub name: Option<String>,
    pub episode_title: Option<String>,
    pub season_numbers: Vec<u32>,
    pub episode_numbers: Vec<u32>,
    pub start_year: Option<u32>,
    pub end_year: Option<u32>,
    pub edition: Option<String>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<u64>,
    pub tvdb_id: Option<u64>,
    pub anidb_id: Option<u64>,
    pub trakt_id: Option<u64>,
    // todo: for things like audio channels, 10bit, etc
    // pub tags: Vec<String>,
}

pub async fn parse_files(file_paths: Vec<String>) -> Vec<ParsedFile> {
    tokio::task::spawn_blocking(move || {
        const BATCH_SIZE: usize = 100;

        let inferred_episode_numbers = infer_numbers::infer_additional_episode_numbers(&file_paths);
        let mut results = Vec::with_capacity(file_paths.len());

        for (chunk_idx, chunk) in file_paths.chunks(BATCH_SIZE).enumerate() {
            let batched_entities = run_batched_inference(&chunk).expect("model inference failed");

            for (inner_idx, entities) in batched_entities.into_iter().enumerate() {
                let global_idx = chunk_idx * BATCH_SIZE + inner_idx;
                let file_name = &chunk[inner_idx];

                let mut c = ParserContext::new();
                for entity in &entities {
                    c.add_match(entity.range());
                }

                let (title, episode_title) = parse_title(&entities);
                let (season_numbers, mut episode_numbers) =
                    parse_season_episode(&file_name, &mut c);
                let (start_year, end_year) = parse_year(&file_name, &mut c);
                let edition = parse_edition(&file_name, &mut c);
                let (imdb_id, tmdb_id, tvdb_id, anidb_id, trakt_id) = parse_ids(&file_name, &mut c);

                if let Some(extra_numbers) = inferred_episode_numbers.get(global_idx) {
                    for &episode in extra_numbers {
                        if !episode_numbers.contains(&episode) {
                            episode_numbers.push(episode);
                        }
                    }
                }

                results.push(ParsedFile {
                    name: title,
                    episode_title,
                    season_numbers,
                    episode_numbers,
                    start_year,
                    end_year,
                    edition,
                    imdb_id,
                    tmdb_id,
                    tvdb_id,
                    anidb_id,
                    trakt_id,
                });
            }
        }

        results
    })
    .await
    .unwrap()
}

pub async fn parse_file(file_name: String) -> ParsedFile {
    let parsed = parse_files(vec![file_name]).await;
    parsed.into_iter().next().unwrap()
}
