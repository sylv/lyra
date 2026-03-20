use crate::entities::{files, node_closure, nodes};
use crate::ids;
use lyra_parser::ParsedFile;
use std::collections::{BTreeSet, HashMap};
use std::path::{Component, Path as StdPath};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WantedNode {
    pub id: String,
    pub root_id: String,
    pub parent_id: Option<String>,
    pub kind: nodes::NodeKind,
    pub name: String,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub last_added_at: i64,
    pub attached_file_ids: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct RootMaterializationPlan {
    pub root_id: String,
    pub wanted_nodes: HashMap<String, WantedNode>,
}

struct FileRecommendation {
    root_id: String,
    root_kind: nodes::NodeKind,
    root_name: String,
    root_imdb_id: Option<String>,
    root_tmdb_id: Option<i64>,
    season: Option<(String, i64, String)>,
    episodes: Vec<(String, i64, String)>,
}

pub fn build_root_materialization_plans(
    library_root: &StdPath,
    input: &[(files::Model, ParsedFile)],
) -> HashMap<String, RootMaterializationPlan> {
    let mut plans = HashMap::new();

    for (file, parsed) in input {
        let Some(rec) = derive_file_recommendation(library_root, &file.relative_path, parsed)
        else {
            continue;
        };

        let plan = plans
            .entry(rec.root_id.clone())
            .or_insert_with(|| RootMaterializationPlan {
                root_id: rec.root_id.clone(),
                wanted_nodes: HashMap::new(),
            });

        ensure_node(
            &mut plan.wanted_nodes,
            WantedNode {
                id: rec.root_id.clone(),
                root_id: rec.root_id.clone(),
                parent_id: None,
                kind: rec.root_kind,
                name: rec.root_name.clone(),
                season_number: None,
                episode_number: None,
                imdb_id: rec.root_imdb_id.clone(),
                tmdb_id: rec.root_tmdb_id,
                last_added_at: file.discovered_at,
                attached_file_ids: Vec::new(),
            },
        );

        if rec.root_kind == nodes::NodeKind::Movie {
            attach_file(&mut plan.wanted_nodes, &rec.root_id, &file.id);
            continue;
        }

        if let Some((season_id, season_number, season_name)) = &rec.season {
            ensure_node(
                &mut plan.wanted_nodes,
                WantedNode {
                    id: season_id.clone(),
                    root_id: rec.root_id.clone(),
                    parent_id: Some(rec.root_id.clone()),
                    kind: nodes::NodeKind::Season,
                    name: season_name.clone(),
                    season_number: Some(*season_number),
                    episode_number: None,
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: file.discovered_at,
                    attached_file_ids: Vec::new(),
                },
            );
        }

        for (episode_id, episode_number, episode_name) in rec.episodes {
            ensure_node(
                &mut plan.wanted_nodes,
                WantedNode {
                    id: episode_id.clone(),
                    root_id: rec.root_id.clone(),
                    parent_id: rec.season.as_ref().map(|(id, _, _)| id.clone()),
                    kind: nodes::NodeKind::Episode,
                    name: episode_name,
                    season_number: rec.season.as_ref().map(|(_, number, _)| *number),
                    episode_number: Some(episode_number),
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: file.discovered_at,
                    attached_file_ids: vec![file.id.clone()],
                },
            );
        }
    }

    plans
}

pub fn group_parsed_files_by_root(
    library_root: &StdPath,
    input: &[(files::Model, ParsedFile)],
) -> HashMap<String, Vec<(files::Model, ParsedFile)>> {
    let mut grouped = HashMap::new();

    for (file, parsed) in input {
        let Some(rec) = derive_file_recommendation(library_root, &file.relative_path, parsed)
        else {
            continue;
        };

        grouped
            .entry(rec.root_id)
            .or_insert_with(Vec::new)
            .push((file.clone(), parsed.clone()));
    }

    grouped
}

fn ensure_node(nodes_by_id: &mut HashMap<String, WantedNode>, next: WantedNode) {
    if let Some(existing) = nodes_by_id.get_mut(&next.id) {
        existing.name = next.name;
        existing.parent_id = next.parent_id;
        existing.kind = next.kind;
        existing.season_number = next.season_number;
        existing.episode_number = next.episode_number;
        existing.last_added_at = existing.last_added_at.max(next.last_added_at);
        if existing.imdb_id.is_none() {
            existing.imdb_id = next.imdb_id;
        }
        if existing.tmdb_id.is_none() {
            existing.tmdb_id = next.tmdb_id;
        }
        for file_id in next.attached_file_ids {
            if !existing.attached_file_ids.contains(&file_id) {
                existing.attached_file_ids.push(file_id);
            }
        }
        return;
    }

    nodes_by_id.insert(next.id.clone(), next);
}

fn attach_file(nodes_by_id: &mut HashMap<String, WantedNode>, node_id: &str, file_id: &str) {
    let Some(node) = nodes_by_id.get_mut(node_id) else {
        return;
    };

    if !node
        .attached_file_ids
        .iter()
        .any(|existing| existing == file_id)
    {
        node.attached_file_ids.push(file_id.to_owned());
    }
}

pub fn sort_nodes_topologically(
    nodes_by_id: &HashMap<String, WantedNode>,
) -> anyhow::Result<Vec<WantedNode>> {
    let mut nodes = nodes_by_id.values().cloned().collect::<Vec<_>>();
    nodes.sort_by(|a, b| {
        node_depth(a, nodes_by_id)
            .cmp(&node_depth(b, nodes_by_id))
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(nodes)
}

fn node_depth(node: &WantedNode, nodes_by_id: &HashMap<String, WantedNode>) -> i64 {
    let mut depth = 0_i64;
    let mut current = node.parent_id.as_ref();
    while let Some(parent_id) = current {
        depth += 1;
        current = nodes_by_id
            .get(parent_id)
            .and_then(|parent| parent.parent_id.as_ref());
    }
    depth
}

pub fn build_closure_rows(
    nodes_by_id: &HashMap<String, WantedNode>,
    descendant_ids: &[String],
) -> anyhow::Result<Vec<node_closure::Model>> {
    let mut rows = Vec::new();

    for descendant_id in descendant_ids {
        let mut current = Some(descendant_id.clone());
        let mut depth = 0_i64;
        while let Some(ancestor_id) = current {
            let Some(cursor) = nodes_by_id.get(&ancestor_id) else {
                anyhow::bail!("closure build missing node {ancestor_id}");
            };

            rows.push(node_closure::Model {
                ancestor_id: ancestor_id.clone(),
                descendant_id: descendant_id.clone(),
                depth,
            });
            current = cursor.parent_id.clone();
            depth += 1;
        }
    }

    rows.sort_by(|a, b| {
        a.ancestor_id
            .cmp(&b.ancestor_id)
            .then_with(|| a.descendant_id.cmp(&b.descendant_id))
    });
    Ok(rows)
}

pub fn verify_root_nodes(nodes: &[WantedNode]) -> anyhow::Result<()> {
    let by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();
    let mut season_keys = BTreeSet::new();
    let mut episode_keys = BTreeSet::new();
    let mut child_kinds: HashMap<String, BTreeSet<nodes::NodeKind>> = HashMap::new();

    for node in nodes {
        match node.kind {
            nodes::NodeKind::Movie | nodes::NodeKind::Series => {
                if node.parent_id.is_some() || node.root_id != node.id {
                    anyhow::bail!("root node {} has invalid parent/root_id", node.id);
                }
            }
            nodes::NodeKind::Season => {
                if node.season_number.is_none() || node.episode_number.is_some() {
                    anyhow::bail!("season node {} has invalid numbers", node.id);
                }
                let parent_id = node
                    .parent_id
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("season {} missing parent", node.id))?;
                let parent = by_id
                    .get(parent_id)
                    .ok_or_else(|| anyhow::anyhow!("season {} missing parent node", node.id))?;
                if parent.kind != nodes::NodeKind::Series {
                    anyhow::bail!("season {} parent must be series", node.id);
                }
                if !season_keys.insert((parent_id.clone(), node.season_number.unwrap())) {
                    anyhow::bail!("duplicate season number under {}", parent_id);
                }
            }
            nodes::NodeKind::Episode => {
                if node.episode_number.is_none() {
                    anyhow::bail!("episode node {} has invalid numbers", node.id);
                }
                let parent_id = node
                    .parent_id
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("episode {} missing parent", node.id))?;
                let parent = by_id
                    .get(parent_id)
                    .ok_or_else(|| anyhow::anyhow!("episode {} missing parent node", node.id))?;
                if !matches!(
                    parent.kind,
                    nodes::NodeKind::Series | nodes::NodeKind::Season
                ) {
                    anyhow::bail!("episode {} parent must be series or season", node.id);
                }
                match parent.kind {
                    nodes::NodeKind::Season => {
                        if node.season_number != parent.season_number {
                            anyhow::bail!(
                                "episode {} season number does not match parent",
                                node.id
                            );
                        }
                    }
                    nodes::NodeKind::Series => {
                        if node.season_number.is_some() {
                            anyhow::bail!(
                                "episode {} under series cannot have season number",
                                node.id
                            );
                        }
                    }
                    _ => {}
                }
                if !episode_keys.insert((parent_id.clone(), node.episode_number.unwrap())) {
                    anyhow::bail!("duplicate episode number under {}", parent_id);
                }
            }
        }

        if let Some(parent_id) = &node.parent_id {
            child_kinds
                .entry(parent_id.clone())
                .or_default()
                .insert(node.kind);
        }

        if let Some(parent_id) = &node.parent_id {
            let mut cursor = by_id
                .get(parent_id)
                .ok_or_else(|| anyhow::anyhow!("missing parent {}", parent_id))?;
            while let Some(next_parent_id) = &cursor.parent_id {
                cursor = by_id
                    .get(next_parent_id)
                    .ok_or_else(|| anyhow::anyhow!("missing ancestor {}", next_parent_id))?;
            }
            if cursor.id != node.root_id {
                anyhow::bail!("node {} root_id does not match top ancestor", node.id);
            }
        }
    }

    for node in nodes {
        let kinds = child_kinds.get(&node.id).cloned().unwrap_or_default();
        if matches!(node.kind, nodes::NodeKind::Movie | nodes::NodeKind::Episode)
            && !kinds.is_empty()
        {
            anyhow::bail!("playable node {} has children", node.id);
        }
        if node.kind == nodes::NodeKind::Series
            && kinds.contains(&nodes::NodeKind::Season)
            && kinds.contains(&nodes::NodeKind::Episode)
        {
            anyhow::bail!("series {} mixes season and episode children", node.id);
        }
    }

    Ok(())
}

fn derive_file_recommendation(
    library_root: &StdPath,
    relative_path: &str,
    parsed: &ParsedFile,
) -> Option<FileRecommendation> {
    let first_dir_past_root_dir = {
        let path = StdPath::new(relative_path);
        let mut components = path.components();
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
    }
    .to_lowercase();

    let season_number = parsed_season_number(parsed);
    let episode_numbers = parsed_episode_numbers(parsed);
    let root_kind = if season_number.is_some() || !episode_numbers.is_empty() {
        nodes::NodeKind::Series
    } else if parsed.start_year.is_some() {
        nodes::NodeKind::Movie
    } else {
        tracing::warn!(relative_path, root = %library_root.display(), "could not determine media kind");
        return None;
    };

    let Some(title) = parsed_title(parsed) else {
        tracing::warn!(relative_path, "file is missing a parsed title");
        return None;
    };

    let root_kind_key = format!("{root_kind:?}");
    let title_key = title.to_lowercase();
    let root_id = ids::generate_hashid([
        first_dir_past_root_dir.as_str(),
        root_kind_key.as_str(),
        title_key.as_str(),
    ]);
    if root_kind == nodes::NodeKind::Movie {
        return Some(FileRecommendation {
            root_id,
            root_kind,
            root_name: title.to_string(),
            root_imdb_id: parsed.imdb_id.clone(),
            root_tmdb_id: parsed.tmdb_id.and_then(|value| i64::try_from(value).ok()),
            season: None,
            episodes: Vec::new(),
        });
    }

    let season = season_number.map(|num| {
        let season_key = format!("season {num}");
        let season_id = ids::generate_hashid([root_id.as_str(), season_key.as_str()]);
        (season_id, i64::from(num), format!("Season {num}"))
    });

    let lineage_id = season
        .as_ref()
        .map(|(id, _, _)| id.clone())
        .unwrap_or_else(|| root_id.clone());

    Some(FileRecommendation {
        root_id,
        root_kind,
        root_name: title.to_string(),
        root_imdb_id: parsed.imdb_id.clone(),
        root_tmdb_id: parsed.tmdb_id.and_then(|value| i64::try_from(value).ok()),
        season,
        episodes: episode_numbers
            .into_iter()
            .map(|episode_number| {
                let episode_key = format!("episode {episode_number}");
                (
                    ids::generate_hashid([lineage_id.as_str(), episode_key.as_str()]),
                    i64::from(episode_number),
                    parsed_episode_name(parsed, episode_number),
                )
            })
            .collect(),
    })
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
        .unwrap_or_else(|| format!("Episode {episode_number}"))
}
