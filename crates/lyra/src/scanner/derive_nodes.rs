use crate::entities::{files, node_closure, node_files, nodes};
use crate::ids;
use lyra_parser::ParsedFile;
use std::collections::{BTreeSet, HashMap};
use std::path::{Component, Path as StdPath};

#[derive(Clone)]
pub struct DerivedNode {
    pub id: String,
    pub root_id: String,
    pub parent_id: Option<String>,
    pub kind: nodes::NodeKind,
    pub name: String,
    pub order: i64,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub last_added_at: i64,
}

#[derive(Clone)]
pub struct DerivedLibraryMedia {
    pub nodes: Vec<DerivedNode>,
    pub node_files: Vec<node_files::Model>,
    pub closure: Vec<node_closure::Model>,
}

#[derive(Clone)]
struct Link {
    file_id: String,
    size_bytes: i64,
}

struct RootAcc {
    id: String,
    kind: nodes::NodeKind,
    name: String,
    imdb_id: Option<String>,
    tmdb_id: Option<i64>,
    last_added_at: i64,
}

struct SeasonAcc {
    id: String,
    root_id: String,
    season_number: i64,
    name: String,
    last_added_at: i64,
}

struct EpisodeAcc {
    id: String,
    root_id: String,
    parent_id: Option<String>,
    episode_number: i64,
    name: String,
    last_added_at: i64,
    links: Vec<Link>,
}

struct MovieAcc {
    id: String,
    name: String,
    imdb_id: Option<String>,
    tmdb_id: Option<i64>,
    last_added_at: i64,
    links: Vec<Link>,
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

pub fn derive_library_media(
    library_root: &StdPath,
    input: &[(files::Model, ParsedFile)],
) -> anyhow::Result<DerivedLibraryMedia> {
    let mut roots: HashMap<String, RootAcc> = HashMap::new();
    let mut seasons: HashMap<String, SeasonAcc> = HashMap::new();
    let mut movies: HashMap<String, MovieAcc> = HashMap::new();
    let mut episodes: HashMap<String, EpisodeAcc> = HashMap::new();

    for (file, parsed) in input {
        let Some(rec) = derive_file_recommendation(library_root, &file.relative_path, parsed)
        else {
            continue;
        };

        let root_entry = roots.entry(rec.root_id.clone()).or_insert_with(|| RootAcc {
            id: rec.root_id.clone(),
            kind: rec.root_kind,
            name: rec.root_name.clone(),
            imdb_id: rec.root_imdb_id.clone(),
            tmdb_id: rec.root_tmdb_id,
            last_added_at: file.discovered_at,
        });
        root_entry.last_added_at = root_entry.last_added_at.max(file.discovered_at);
        if root_entry.imdb_id.is_none() {
            root_entry.imdb_id = rec.root_imdb_id.clone();
        }
        if root_entry.tmdb_id.is_none() {
            root_entry.tmdb_id = rec.root_tmdb_id;
        }

        if rec.root_kind == nodes::NodeKind::Movie {
            let movie = movies
                .entry(rec.root_id.clone())
                .or_insert_with(|| MovieAcc {
                    id: rec.root_id.clone(),
                    name: rec.root_name.clone(),
                    imdb_id: rec.root_imdb_id.clone(),
                    tmdb_id: rec.root_tmdb_id,
                    last_added_at: file.discovered_at,
                    links: Vec::new(),
                });
            movie.last_added_at = movie.last_added_at.max(file.discovered_at);
            upsert_link(&mut movie.links, file);
            continue;
        }

        if let Some((season_id, season_number, season_name)) = &rec.season {
            let season = seasons
                .entry(season_id.clone())
                .or_insert_with(|| SeasonAcc {
                    id: season_id.clone(),
                    root_id: rec.root_id.clone(),
                    season_number: *season_number,
                    name: season_name.clone(),
                    last_added_at: file.discovered_at,
                });
            season.last_added_at = season.last_added_at.max(file.discovered_at);
        }

        for (episode_id, episode_number, name) in rec.episodes {
            let episode = episodes
                .entry(episode_id.clone())
                .or_insert_with(|| EpisodeAcc {
                    id: episode_id.clone(),
                    root_id: rec.root_id.clone(),
                    parent_id: rec.season.as_ref().map(|(id, _, _)| id.clone()),
                    episode_number,
                    name,
                    last_added_at: file.discovered_at,
                    links: Vec::new(),
                });
            episode.last_added_at = episode.last_added_at.max(file.discovered_at);
            upsert_link(&mut episode.links, file);
        }
    }

    movies.retain(|_, movie| !movie.links.is_empty());
    episodes.retain(|_, episode| !episode.links.is_empty());

    let mut nodes = Vec::new();
    let mut node_file_rows = Vec::new();

    let mut root_ids = roots.keys().cloned().collect::<BTreeSet<_>>();
    root_ids.extend(movies.keys().cloned());

    for root_id in root_ids {
        let Some(root) = roots.get(&root_id) else {
            continue;
        };

        nodes.push(DerivedNode {
            id: root.id.clone(),
            root_id: root.id.clone(),
            parent_id: None,
            kind: root.kind,
            name: root.name.clone(),
            order: 0,
            season_number: None,
            episode_number: None,
            imdb_id: root.imdb_id.clone(),
            tmdb_id: root.tmdb_id,
            last_added_at: root.last_added_at,
        });

        if root.kind == nodes::NodeKind::Movie {
            let Some(movie) = movies.get(&root.id) else {
                continue;
            };
            for (order, link) in sorted_links(&movie.links).into_iter().enumerate() {
                node_file_rows.push(node_files::Model {
                    node_id: movie.id.clone(),
                    file_id: link.file_id,
                    order: order as i64,
                    created_at: 0,
                    updated_at: 0,
                });
            }
            continue;
        }

        let mut order = 1_i64;
        let mut root_seasons = seasons
            .values()
            .filter(|season| season.root_id == root.id)
            .collect::<Vec<_>>();
        root_seasons.sort_by(|a, b| {
            a.season_number
                .cmp(&b.season_number)
                .then_with(|| a.id.cmp(&b.id))
        });

        let mut root_episodes = episodes
            .values()
            .filter(|episode| episode.root_id == root.id)
            .collect::<Vec<_>>();
        root_episodes.sort_by(|a, b| {
            a.episode_number
                .cmp(&b.episode_number)
                .then_with(|| a.id.cmp(&b.id))
        });

        if !root_seasons.is_empty() {
            for season in root_seasons {
                nodes.push(DerivedNode {
                    id: season.id.clone(),
                    root_id: root.id.clone(),
                    parent_id: Some(root.id.clone()),
                    kind: nodes::NodeKind::Season,
                    name: season.name.clone(),
                    order,
                    season_number: Some(season.season_number),
                    episode_number: None,
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: season.last_added_at,
                });
                order += 1;

                let mut season_episodes = episodes
                    .values()
                    .filter(|episode| episode.parent_id.as_deref() == Some(season.id.as_str()))
                    .collect::<Vec<_>>();
                season_episodes.sort_by(|a, b| {
                    a.episode_number
                        .cmp(&b.episode_number)
                        .then_with(|| a.id.cmp(&b.id))
                });

                for episode in season_episodes {
                    nodes.push(DerivedNode {
                        id: episode.id.clone(),
                        root_id: root.id.clone(),
                        parent_id: Some(season.id.clone()),
                        kind: nodes::NodeKind::Episode,
                        name: episode.name.clone(),
                        order,
                        season_number: Some(season.season_number),
                        episode_number: Some(episode.episode_number),
                        imdb_id: None,
                        tmdb_id: None,
                        last_added_at: episode.last_added_at,
                    });
                    order += 1;

                    for (file_order, link) in sorted_links(&episode.links).into_iter().enumerate() {
                        node_file_rows.push(node_files::Model {
                            node_id: episode.id.clone(),
                            file_id: link.file_id,
                            order: file_order as i64,
                            created_at: 0,
                            updated_at: 0,
                        });
                    }
                }
            }
        } else {
            for episode in root_episodes {
                nodes.push(DerivedNode {
                    id: episode.id.clone(),
                    root_id: root.id.clone(),
                    parent_id: Some(root.id.clone()),
                    kind: nodes::NodeKind::Episode,
                    name: episode.name.clone(),
                    order,
                    season_number: None,
                    episode_number: Some(episode.episode_number),
                    imdb_id: None,
                    tmdb_id: None,
                    last_added_at: episode.last_added_at,
                });
                order += 1;

                for (file_order, link) in sorted_links(&episode.links).into_iter().enumerate() {
                    node_file_rows.push(node_files::Model {
                        node_id: episode.id.clone(),
                        file_id: link.file_id,
                        order: file_order as i64,
                        created_at: 0,
                        updated_at: 0,
                    });
                }
            }
        }
    }

    // keep inserts topological so sqlite's immediate node/root foreign keys always see parents first
    nodes.sort_by(|a, b| {
        a.root_id
            .cmp(&b.root_id)
            .then_with(|| a.order.cmp(&b.order))
            .then_with(|| a.id.cmp(&b.id))
    });
    node_file_rows.sort_by(|a, b| {
        a.node_id
            .cmp(&b.node_id)
            .then_with(|| a.file_id.cmp(&b.file_id))
    });

    let closure = build_closure_rows(&nodes)?;
    verify_nodes(&nodes, &node_file_rows, &closure)?;

    Ok(DerivedLibraryMedia {
        nodes,
        node_files: node_file_rows,
        closure,
    })
}

fn upsert_link(links: &mut Vec<Link>, file: &files::Model) {
    if let Some(link) = links.iter_mut().find(|link| link.file_id == file.id) {
        link.size_bytes = file.size_bytes;
    } else {
        links.push(Link {
            file_id: file.id.clone(),
            size_bytes: file.size_bytes,
        });
    }
}

fn sorted_links(links: &[Link]) -> Vec<Link> {
    let mut links = links.to_vec();
    links.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.file_id.cmp(&b.file_id))
    });
    links
}

fn build_closure_rows(nodes: &[DerivedNode]) -> anyhow::Result<Vec<node_closure::Model>> {
    let by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();
    let mut rows = Vec::new();

    for node in nodes {
        let mut current = Some(node.id.clone());
        let mut depth = 0_i64;
        while let Some(descendant_id) = current {
            let Some(cursor) = by_id.get(&descendant_id) else {
                anyhow::bail!("closure build missing node {descendant_id}");
            };

            rows.push(node_closure::Model {
                ancestor_id: descendant_id.clone(),
                descendant_id: node.id.clone(),
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

pub fn verify_nodes(
    nodes: &[DerivedNode],
    node_files: &[node_files::Model],
    closure: &[node_closure::Model],
) -> anyhow::Result<()> {
    let by_id = nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<HashMap<_, _>>();
    let mut seen_root_order = BTreeSet::new();
    let mut season_keys = BTreeSet::new();
    let mut episode_keys = BTreeSet::new();
    let mut child_kinds: HashMap<String, BTreeSet<nodes::NodeKind>> = HashMap::new();

    for node in nodes {
        if !seen_root_order.insert((node.root_id.clone(), node.order)) {
            anyhow::bail!("duplicate root/order for node {}", node.id);
        }

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
                // episodes under seasons carry both season and episode numbers, while episodes
                // directly under a series only carry an episode number.
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

    for link in node_files {
        let Some(node) = by_id.get(&link.node_id) else {
            anyhow::bail!("node_files references missing node {}", link.node_id);
        };
        if !matches!(node.kind, nodes::NodeKind::Movie | nodes::NodeKind::Episode) {
            anyhow::bail!("node_files references non-playable node {}", node.id);
        }
    }

    let closure_set = closure
        .iter()
        .map(|row| {
            (
                row.ancestor_id.clone(),
                row.descendant_id.clone(),
                row.depth,
            )
        })
        .collect::<BTreeSet<_>>();
    for node in nodes {
        if !closure_set.contains(&(node.id.clone(), node.id.clone(), 0)) {
            anyhow::bail!("node {} missing self closure row", node.id);
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
