use crate::model::Entity;
use std::collections::HashMap;

pub fn parse_title(entities: &[Entity]) -> (Option<String>, Option<String>) {
    let name_entities = entities
        .iter()
        .filter(|e| e.label == "NAME")
        .filter_map(|e| {
            let trimmed = trim_title(&e.text);
            if trimmed.is_empty() {
                None
            } else {
                Some((e.start, trimmed))
            }
        })
        .collect::<Vec<_>>();

    if name_entities.is_empty() {
        return (None, None);
    }

    let title = select_title(&name_entities);

    let last_episode_title = entities
        .iter()
        .filter(|e| e.label == "EPISODE_TITLE")
        .last()
        .map(|e| trim_title(&e.text));

    (Some(title), last_episode_title)
}

fn select_title(name_entities: &[(usize, String)]) -> String {
    let mut counts = HashMap::<String, usize>::new();
    for (_, title) in name_entities {
        *counts.entry(normalize_title_key(title)).or_insert(0) += 1;
    }

    let max_count = counts.values().copied().max().unwrap_or(0);
    let top_keys = counts
        .iter()
        .filter(|(_, count)| **count == max_count)
        .map(|(key, _)| key.as_str())
        .collect::<Vec<_>>();

    // If one title is clearly the most common, prefer it. Otherwise, use the earliest one in the path.
    if max_count > 1 && top_keys.len() == 1 {
        let majority_key = top_keys[0];
        if let Some((_, title)) = name_entities
            .iter()
            .find(|(_, title)| normalize_title_key(title) == majority_key)
        {
            return title.clone();
        }
    }

    name_entities
        .iter()
        .min_by_key(|(start, _)| *start)
        .map(|(_, title)| title.clone())
        .expect("name_entities is non-empty")
}

fn normalize_title_key(title: &str) -> String {
    title.to_ascii_lowercase()
}

fn trim_title(title: &str) -> String {
    // strip - from the start, ] from the start, [ from the end
    // replace . and _ with spaces
    // trim trailing/leading whitespace
    // todo: some of this is likely due to tokenization issues in the model. the training data is index-based which
    // then have to be aligned to labels and its probably not perfect when it does that, causing `]` to be included in the title.
    // or `]` is just included in the title tokens, but i doubt it.
    let title = title.replace('.', " ").replace('_', " ");
    let title = title.trim();
    let title = title.trim_start_matches('-');
    let title = title.trim_start_matches(']');
    let title = title.trim_end_matches('[');
    title.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(start: usize, text: &str) -> Entity {
        Entity {
            label: "NAME".to_string(),
            start,
            end: start + text.len(),
            text: text.to_string(),
        }
    }

    #[test]
    fn test_majority_title_wins() {
        let entities = vec![
            name(0, "Gen V"),
            name(40, "The Whole Truth"),
            name(70, "Gen V"),
        ];

        let (title, episode_title) = parse_title(&entities);
        assert_eq!(title, Some("Gen V".to_string()));
        assert_eq!(episode_title, None);
    }

    #[test]
    fn test_tie_uses_earliest_title() {
        let entities = vec![name(40, "Steins;Gate -"), name(0, "Steins;Gate")];

        let (title, episode_title) = parse_title(&entities);
        assert_eq!(title, Some("Steins;Gate".to_string()));
        assert_eq!(episode_title, None);
    }
}
