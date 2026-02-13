use crate::entities::nodes::NodeKind;
use lyra_parser::ParsedFile;
use sha2::{Digest, Sha256};
use std::path::{Component, Path as StdPath};

pub struct NodeRecommendation {
    pub id: String,
    pub root_id: Option<String>,
    pub parent_id: Option<String>,
    pub kind: NodeKind,
    pub name: String,
    pub attach_file: bool,
    pub episode_number: Option<i32>,
}

pub fn get_recommended_nodes_for_file(
    library_root: &StdPath,
    file_path: &StdPath,
    parsed: &ParsedFile,
) -> Option<Vec<NodeRecommendation>> {
    // given eg "placeholder.mkv", we get "Inception (2010)"
    // given eg "placeholder.mkv" this gives us an empty string
    let first_dir_past_root_dir = {
        if let Ok(stripped) = file_path.strip_prefix(library_root) {
            let mut components = stripped.components();
            match components.next() {
                Some(Component::Normal(first)) => {
                    if components.next().is_some() {
                        first.to_string_lossy().into_owned()
                    } else {
                        String::new()
                    }
                }
                _ => String::new(),
            }
        } else {
            String::new()
        }
    }
    .to_lowercase();

    // infer whether its a movie or series from whether season_num/episode_num is present
    let season_number = parsed_season_number(parsed);
    let episode_numbers = parsed_episode_numbers(parsed);
    let root_kind = if season_number.is_some() || !episode_numbers.is_empty() {
        NodeKind::Series
    } else if parsed.start_year.is_some() {
        // movies must have a year, just a name isnt enough for reliable matching
        NodeKind::Movie
    } else {
        tracing::warn!(
            "could not determine whether {} is a movie or series, skipping!",
            file_path.display()
        );
        return None;
    };

    let Some(title) = parsed_title(parsed) else {
        tracing::warn!(
            "file {} is missing a parsed title, skipping!",
            file_path.display()
        );
        return None;
    };

    let lower_title = title.to_lowercase();
    let kind_str = format!("{:?}", root_kind);
    let root_id = get_node_id_from_hash_of(&[&first_dir_past_root_dir, &kind_str, &lower_title]);

    let mut recommended_nodes = vec![NodeRecommendation {
        id: root_id.clone(),
        root_id: None,
        parent_id: None,
        kind: root_kind,
        name: title.to_string(),
        attach_file: root_kind == NodeKind::Movie,
        episode_number: None,
    }];

    if root_kind != NodeKind::Movie {
        if episode_numbers.is_empty() {
            tracing::warn!(
                "file {} was detected as an episode file but did not contain any parsed episode numbers",
                file_path.display()
            );
            return None;
        }

        let parent_id = if let Some(season_number) = season_number {
            let season_number_str = format!("season {}", season_number);
            let season_id = get_node_id_from_hash_of(&[&root_id, &season_number_str]);
            recommended_nodes.push(NodeRecommendation {
                id: season_id.clone(),
                root_id: Some(root_id.clone()),
                parent_id: Some(root_id.clone()),
                kind: NodeKind::Season,
                name: format!("Season {}", season_number),
                attach_file: false,
                episode_number: None,
            });

            season_id
        } else {
            root_id.clone()
        };

        for ep_num in episode_numbers {
            let ep_num_str = format!("episode {}", ep_num);
            let episode_id = get_node_id_from_hash_of(&[&parent_id, &ep_num_str]);
            recommended_nodes.push(NodeRecommendation {
                id: episode_id,
                root_id: Some(root_id.clone()),
                parent_id: Some(parent_id.clone()),
                kind: NodeKind::Episode,
                name: parsed_episode_name(parsed, ep_num),
                attach_file: true,
                episode_number: Some(ep_num),
            });
        }
    }

    Some(recommended_nodes)
}

fn parsed_title(parsed: &ParsedFile) -> Option<&str> {
    parsed.name.as_deref().and_then(|name| {
        let trimmed = name.trim();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn parsed_season_number(parsed: &ParsedFile) -> Option<i32> {
    parsed
        .season_numbers
        .iter()
        .filter_map(|&season| i32::try_from(season).ok())
        .min()
}

fn parsed_episode_numbers(parsed: &ParsedFile) -> Vec<i32> {
    let mut episodes = parsed
        .episode_numbers
        .iter()
        .filter_map(|&episode| i32::try_from(episode).ok())
        .collect::<Vec<_>>();
    episodes.sort_unstable();
    episodes.dedup();
    episodes
}

fn parsed_episode_name(parsed: &ParsedFile, episode_number: i32) -> String {
    parsed
        .episode_title
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("Episode {}", episode_number))
}

fn get_node_id_from_hash_of(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.to_lowercase().as_bytes());
    }
    let result = hasher.finalize();
    let hex = hex::encode(result);
    format!("n_{}", &hex[..16])
}
