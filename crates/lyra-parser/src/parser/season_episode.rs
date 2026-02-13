use crate::pattern;
use crate::util::parse_possible_range;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

lazy_static! {
    static ref SEASON_EPISODE_PATTERNS: Vec<Regex> = vec![
        // "some_title_ep03.mp4"
        pattern!(r"(?i)ep(?P<episode>[0-9]{1,2})\b"),
        // "Season 1/01", "SE1/01", "S1/01" - season/episode in path
        pattern!(r"(?i)(?:Season|SE?)\s*(?P<season>[0-9]{1,2})/(?P<episode>[0-9]{1,2})"),
        // "season 1", "season 1 episode 1", "season 1/episode 1", "season.1"
        pattern!(r"(?i)Season[ .](?P<season>[0-9]{1,2})(( |/)Episode (?P<episode>[0-9]{1,2}))?"),
        // "Season 1 to 6", "Seasons 1 to 6"
        pattern!(r"(?i)Seasons? (?P<season>[0-9]{1,2} to [0-9]{1,2})"),
        // "1x1", "1x12", "7x23-24"
        pattern!(r"(?i)(\[|\(|\b)(?P<season>[0-9]{1,2})x(?P<episode>[0-9]{1,2}(?:-[0-9]{1,2})*)(\]|\)|\b)"),
        // "S01E01", "S01 E02", "SE01EP01", "S01.E01", "S03E12-E13"
        pattern!(r"(?i)SE?(?P<season>[0-9]{1,3})[ .]?EP?(?P<episode>[0-9]{1,3}(?:(?:-E?|E)[0-9]{1,3})*)"),
        // "S01", standalone season notation
        pattern!(r"(?i)(\b|^)S(?P<season>[0-9]+)(\b|\.|[^0-9])"),
        // "episodes 1-4", "ep1-4", "00~12"
        pattern!(r"(?i)\b(episodes?|ep|e) ?(?P<episode>[0-9~-]{2,})\b"),
        // "S2 - 00~12" format
        pattern!(r"(?i)S(?P<season>[0-9]+) - (?P<episode>[0-9]{1,2}~[0-9]{1,2})"),
        // "season 1-4", "se1-4", "Seasons 1-7", "S01-S02", "SE 1 - 6"
        pattern!(r"(?i)\b(?:seasons?|se|s)\s*(?P<season>[0-9]{1,2}(?:\s*-\s*S?[0-9]{1,2})+)\b"),
        // "Seasons 1 2 3"
        pattern!(r"(?i)\b(?:seasons?|season|se|s) ?(?P<season>[0-9]{1,2}(?:[ ]+[0-9]{1,2})+)\b"),
        // "S01E01E02", "S01E01-E02", "S01E01 - E02", "S01E01-02"
        pattern!(r"(?i)([0-9]|\b)EP?(?P<episode>[0-9]+(?:(?:-E?|E)[0-9]{1,2})+)"),
        // "EP(01-08)", "EP(01)"
        pattern!(r"(?i)(?:ep|episodes) ?(?:\(|\[)(?P<episode>[0-9]{1,3}(?:[ -]+[0-9]{1,3})?)(?:\)|\])"),
        // "One Piece - 1023", "Fairy Tail - 05"
        pattern!(r"(?i) - (?P<episode>[0-9]{1,4})(?: |\(|\[|$)"),
        // "E10 - E17", "EP01 - EP03"
        pattern!(r"(?i)\bE(?:P)?(?P<episode>[0-9]+ - E(?:P)?[0-9]+)\b"),
        // "Episode 3", "Episode 12" - be more specific to avoid model conflicts
        pattern!(r"(?i)/Episode (?P<episode>[0-9]+)\b"),
        // "cap.213", "2" is season and "13" is episode
        pattern!(r"(?i)cap\.(?P<season>\d{1,2})(?P<episode>\d{2})\b"),
        // [CONAN][119]
        pattern!(r"\[(?P<episode>[0-9]{2,3})\]")
    ];
}

#[derive(Debug, Clone)]
struct Match {
    range: core::ops::Range<usize>,
    season_numbers: Vec<u32>,
    episode_numbers: Vec<u32>,
}

/// Parse season and episode numbers from a string
/// Returns (is_season_pack, season_numbers, episode_numbers)
/// Algorithm:
/// - prioritizes rightmost matches (filename over path)
/// - uses segment-based (slash-separated) matching, once a segment contains a valid match only other matches from that same segment are considered
/// - detects season packs when season numbers exist and episodes (if any) are in the final segment
/// - filters out episode numbers over 1500 to avoid mistaking years for episode numbers
/// - when patterns overlap, prefers longer matches
pub fn parse_season_episode(
    normalized: &str,
    context: &mut crate::ParserContext,
) -> (Vec<u32>, Vec<u32>) {
    let mut all_matches: Vec<Match> = Vec::new();

    // Extract all matches from all patterns
    for pattern in SEASON_EPISODE_PATTERNS.iter() {
        for captures in pattern.captures_iter(normalized) {
            let full_match = captures.get(0).unwrap();

            let mut season_numbers = Vec::new();
            let mut episode_numbers = Vec::new();

            if let Some(season_number) = captures.name("season") {
                if !context.overlaps_any(&season_number.range()) {
                    let parsed = parse_possible_range(season_number.as_str(), false);
                    season_numbers.extend(parsed);
                }
            }

            if let Some(episode_number) = captures.name("episode") {
                if !context.overlaps_any(&episode_number.range()) {
                    let parsed = parse_possible_range(episode_number.as_str(), false);
                    // Filter out episode numbers over 1500 to avoid mistaking years for episodes
                    episode_numbers.extend(parsed.into_iter().filter(|&ep| ep <= 1500));
                }
            }

            if !season_numbers.is_empty() || !episode_numbers.is_empty() {
                all_matches.push(Match {
                    range: full_match.range(),
                    season_numbers,
                    episode_numbers,
                });
            }
        }
    }

    // Remove overlapping matches, keeping longer ones
    let mut filtered_matches = Vec::new();
    all_matches.sort_by(|a, b| a.range.start.cmp(&b.range.start));

    for current_match in all_matches {
        let mut should_keep = true;
        filtered_matches.retain(|existing: &Match| {
            let overlaps = current_match.range.start < existing.range.end
                && existing.range.start < current_match.range.end;

            if overlaps {
                let current_len = current_match.range.len();
                let existing_len = existing.range.len();

                if current_len > existing_len {
                    // Current match is longer, remove existing
                    false
                } else {
                    // Existing match is longer or equal, don't add current
                    should_keep = false;
                    true
                }
            } else {
                true
            }
        });

        if should_keep {
            context.add_match(current_match.range.clone());
            filtered_matches.push(current_match);
        }
    }

    // Sort matches by position (rightmost first)
    filtered_matches.sort_by(|a, b| b.range.start.cmp(&a.range.start));

    let mut season_numbers = HashSet::new();
    let mut episode_numbers = HashSet::new();

    // Process season matches with segment-based logic
    let mut require_range: Option<core::ops::Range<usize>> = None;
    for match_info in &filtered_matches {
        if match_info.season_numbers.is_empty() {
            continue;
        }

        if let Some(season_segment) = &require_range {
            if !season_segment.contains(&match_info.range.start) {
                continue;
            }
        } else {
            let previous_slash = normalized[..match_info.range.start].rfind('/').unwrap_or(0);
            let next_slash = normalized[match_info.range.end..]
                .find('/')
                .map(|s| s + match_info.range.end)
                .unwrap_or(normalized.len());

            require_range = Some(previous_slash..next_slash);
        }

        season_numbers.extend(&match_info.season_numbers);
    }

    // Process episode matches with segment-based logic
    let mut require_range: Option<core::ops::Range<usize>> = None;
    for match_info in &filtered_matches {
        if match_info.episode_numbers.is_empty() {
            continue;
        }

        if let Some(episode_segment) = &require_range {
            if !episode_segment.contains(&match_info.range.start) {
                continue;
            }
        } else {
            let previous_slash = normalized[..match_info.range.start].rfind('/').unwrap_or(0);
            let next_slash = normalized[match_info.range.end..]
                .find('/')
                .map(|s| s + match_info.range.end)
                .unwrap_or(normalized.len());

            require_range = Some(previous_slash..next_slash);
        }

        episode_numbers.extend(&match_info.episode_numbers);
    }

    #[allow(unused_mut)]
    let mut season_numbers = season_numbers.into_iter().collect::<Vec<u32>>();
    #[allow(unused_mut)]
    let mut episode_numbers = episode_numbers.into_iter().collect::<Vec<u32>>();

    // Add all matched ranges to context
    for match_info in &filtered_matches {
        context.add_match(match_info.range.clone());
    }

    #[cfg(test)]
    {
        season_numbers.sort();
        episode_numbers.sort();
    }

    (season_numbers, episode_numbers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParserContext;

    #[test]
    fn test_bottom_gear_ranges() {
        let mut context = ParserContext::new();
        let file_name = "(auto) Bottom Gear - 2008 - [11x01] - 2008.06.22 [$2k lambo].avi";
        let (seasons, episodes) = parse_season_episode(file_name, &mut context);

        println!("File: {}", file_name);
        println!("Seasons: {:?}, Episodes: {:?}", seasons, episodes);
        println!("Context ranges: {:?}", context.matched_ranges);

        // Find the actual position of 2008.06.22
        let date_pos = file_name.find("2008.06.22").unwrap();
        let date_range = date_pos..date_pos + 10;
        println!(
            "Date range 2008.06.22: {:?} (actual: '{}')",
            date_range,
            &file_name[date_range.clone()]
        );
        println!(
            "Overlaps with any range: {}",
            context.overlaps_any(&date_range)
        );

        assert_eq!(seasons, vec![11]);
        assert_eq!(episodes, vec![1]);
    }

    #[test]
    fn test_vi_seasons_ranges() {
        let mut context = ParserContext::new();
        let file_name = "[www.example.org] Vi Seasons 1-6/Season 5/Episode 3.mp4";
        let (seasons, episodes) = parse_season_episode(file_name, &mut context);

        println!("File: {}", file_name);
        println!("Seasons: {:?}, Episodes: {:?}", seasons, episodes);
        println!("Context ranges: {:?}", context.matched_ranges);

        assert_eq!(seasons, vec![5]);
        assert_eq!(episodes, vec![3]);
    }

    #[test]
    fn test_peacemaker_episode() {
        let mut context = ParserContext::new();
        let file_name = "Peacemaker S02E01 1080p WEB-DL HEVC x265-RMTeam/Peacemaker S02E01 1080p WEB-DL HEVC x265-RMTeam.mkv";
        let (seasons, episodes) = parse_season_episode(file_name, &mut context);

        println!("File: {}", file_name);
        println!("Seasons: {:?}, Episodes: {:?}", seasons, episodes);
        println!("Context ranges: {:?}", context.matched_ranges);

        let first_slash_idx = file_name.find('/').unwrap();
        println!("First slash at: {}", first_slash_idx);

        assert_eq!(seasons, vec![2]);
        assert_eq!(episodes, vec![1]);
    }
}
