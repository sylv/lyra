use crate::entities::{
    files,
    items::ItemKind,
    roots::RootKind,
};
use lyra_parser::ParsedFile;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path as StdPath};

#[derive(Clone)]
pub struct DerivedRoot {
    pub id: String,
    pub kind: RootKind,
    pub name: String,
    pub last_added_at: i64,
}

#[derive(Clone)]
pub struct DerivedSeason {
    pub id: String,
    pub root_id: String,
    pub season_number: i64,
    pub order: i64,
    pub name: String,
    pub last_added_at: i64,
}

#[derive(Clone)]
pub struct DerivedItem {
    pub id: String,
    pub root_id: String,
    pub season_id: Option<String>,
    pub kind: ItemKind,
    pub episode_number: Option<i64>,
    pub order: i64,
    pub name: String,
    pub primary_file_id: Option<i64>,
    pub last_added_at: i64,
}

#[derive(Clone)]
pub struct DerivedItemFile {
    pub item_id: String,
    pub file_id: i64,
    pub order: i64,
    pub is_primary: bool,
}

pub struct DerivedLibraryMedia {
    pub roots: Vec<DerivedRoot>,
    pub seasons: Vec<DerivedSeason>,
    pub items: Vec<DerivedItem>,
    pub item_files: Vec<DerivedItemFile>,
}

#[derive(Clone)]
struct Link {
    file_id: i64,
    size_bytes: i64,
    discovered_at: i64,
}

struct RootAcc {
    id: String,
    kind: RootKind,
    name: String,
    last_added_at: i64,
}

struct SeasonAcc {
    id: String,
    root_id: String,
    season_number: i64,
    name: String,
    last_added_at: i64,
}

struct ItemAcc {
    id: String,
    root_id: String,
    season_id: Option<String>,
    kind: ItemKind,
    episode_number: Option<i64>,
    name: String,
    last_added_at: i64,
    links: Vec<Link>,
}

struct FileRecommendation {
    root_id: String,
    root_kind: RootKind,
    root_name: String,
    season: Option<(String, i64, String)>,
    items: Vec<(String, ItemKind, Option<i64>, String)>,
}

pub fn derive_library_media(
    library_root: &StdPath,
    input: &[(files::Model, ParsedFile)],
) -> DerivedLibraryMedia {
    let mut roots: HashMap<String, RootAcc> = HashMap::new();
    let mut seasons: HashMap<String, SeasonAcc> = HashMap::new();
    let mut items: HashMap<String, ItemAcc> = HashMap::new();

    for (file, parsed) in input {
        let Some(rec) = derive_file_recommendation(library_root, &file.relative_path, parsed) else {
            continue;
        };

        let root_entry = roots.entry(rec.root_id.clone()).or_insert_with(|| RootAcc {
            id: rec.root_id.clone(),
            kind: rec.root_kind,
            name: rec.root_name.clone(),
            last_added_at: file.discovered_at,
        });
        root_entry.last_added_at = root_entry.last_added_at.max(file.discovered_at);

        if let Some((season_id, season_number, season_name)) = &rec.season {
            let season_entry = seasons.entry(season_id.clone()).or_insert_with(|| SeasonAcc {
                id: season_id.clone(),
                root_id: rec.root_id.clone(),
                season_number: *season_number,
                name: season_name.clone(),
                last_added_at: file.discovered_at,
            });
            season_entry.last_added_at = season_entry.last_added_at.max(file.discovered_at);
        }

        for (item_id, kind, episode_number, name) in rec.items {
            let season_id = rec.season.as_ref().map(|(id, _, _)| id.clone());
            let item_entry = items.entry(item_id.clone()).or_insert_with(|| ItemAcc {
                id: item_id.clone(),
                root_id: rec.root_id.clone(),
                season_id,
                kind,
                episode_number,
                name,
                last_added_at: file.discovered_at,
                links: Vec::new(),
            });

            item_entry.last_added_at = item_entry.last_added_at.max(file.discovered_at);
            if let Some(existing_link) = item_entry.links.iter_mut().find(|link| link.file_id == file.id)
            {
                existing_link.size_bytes = file.size_bytes;
                existing_link.discovered_at = file.discovered_at;
            } else {
                item_entry.links.push(Link {
                    file_id: file.id,
                    size_bytes: file.size_bytes,
                    discovered_at: file.discovered_at,
                });
            }
        }
    }

    // Remove malformed accumulated items (items without file links).
    items.retain(|_, item| !item.links.is_empty());

    // Recompute root and season rollups from item links.
    let mut root_last_added: HashMap<String, i64> = HashMap::new();
    let mut season_last_added: HashMap<String, i64> = HashMap::new();
    for item in items.values() {
        root_last_added
            .entry(item.root_id.clone())
            .and_modify(|value| *value = (*value).max(item.last_added_at))
            .or_insert(item.last_added_at);

        if let Some(season_id) = &item.season_id {
            season_last_added
                .entry(season_id.clone())
                .and_modify(|value| *value = (*value).max(item.last_added_at))
                .or_insert(item.last_added_at);
        }
    }

    roots.retain(|id, root| {
        if let Some(last_added) = root_last_added.get(id) {
            root.last_added_at = *last_added;
            true
        } else {
            false
        }
    });

    seasons.retain(|id, season| {
        if let Some(last_added) = season_last_added.get(id) {
            season.last_added_at = *last_added;
            true
        } else {
            false
        }
    });

    let mut season_order_lookup: HashMap<String, i64> = HashMap::new();
    let mut seasons_by_root: BTreeMap<String, Vec<&SeasonAcc>> = BTreeMap::new();
    for season in seasons.values() {
        seasons_by_root
            .entry(season.root_id.clone())
            .or_default()
            .push(season);
    }

    let mut derived_seasons = Vec::new();
    for root_seasons in seasons_by_root.values_mut() {
        root_seasons.sort_by(|a, b| {
            a.season_number
                .cmp(&b.season_number)
                .then_with(|| a.id.cmp(&b.id))
        });

        for (index, season) in root_seasons.iter().enumerate() {
            let order = index as i64;
            season_order_lookup.insert(season.id.clone(), order);
            derived_seasons.push(DerivedSeason {
                id: season.id.clone(),
                root_id: season.root_id.clone(),
                season_number: season.season_number,
                order,
                name: season.name.clone(),
                last_added_at: season.last_added_at,
            });
        }
    }

    let mut items_by_root: BTreeMap<String, Vec<&ItemAcc>> = BTreeMap::new();
    for item in items.values() {
        items_by_root
            .entry(item.root_id.clone())
            .or_default()
            .push(item);
    }

    let mut derived_items = Vec::new();
    let mut derived_item_files = Vec::new();

    for root_items in items_by_root.values_mut() {
        root_items.sort_by(|a, b| {
            item_sort_key(a, &season_order_lookup)
                .cmp(&item_sort_key(b, &season_order_lookup))
                .then_with(|| a.id.cmp(&b.id))
        });

        for (index, item) in root_items.iter().enumerate() {
            let mut links = item.links.clone();
            links.sort_by(|a, b| {
                item_file_order_for_size(b.size_bytes)
                    .cmp(&item_file_order_for_size(a.size_bytes))
                    .then_with(|| a.file_id.cmp(&b.file_id))
            });

            let primary_file_id = links.first().map(|link| link.file_id);
            for (link_index, link) in links.iter().enumerate() {
                derived_item_files.push(DerivedItemFile {
                    item_id: item.id.clone(),
                    file_id: link.file_id,
                    order: item_file_order_for_size(link.size_bytes),
                    is_primary: link_index == 0,
                });
            }

            derived_items.push(DerivedItem {
                id: item.id.clone(),
                root_id: item.root_id.clone(),
                season_id: item.season_id.clone(),
                kind: item.kind,
                episode_number: item.episode_number,
                order: index as i64,
                name: item.name.clone(),
                primary_file_id,
                last_added_at: item.last_added_at,
            });
        }
    }

    let mut derived_roots = roots
        .into_values()
        .map(|root| DerivedRoot {
            id: root.id,
            kind: root.kind,
            name: root.name,
            last_added_at: root.last_added_at,
        })
        .collect::<Vec<_>>();

    derived_roots.sort_by(|a, b| a.id.cmp(&b.id));
    derived_seasons.sort_by(|a, b| a.id.cmp(&b.id));
    derived_items.sort_by(|a, b| a.id.cmp(&b.id));
    derived_item_files.sort_by(|a, b| a.item_id.cmp(&b.item_id).then_with(|| a.file_id.cmp(&b.file_id)));

    DerivedLibraryMedia {
        roots: derived_roots,
        seasons: derived_seasons,
        items: derived_items,
        item_files: derived_item_files,
    }
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
        RootKind::Series
    } else if parsed.start_year.is_some() {
        RootKind::Movie
    } else {
        tracing::warn!(
            relative_path,
            root = %library_root.display(),
            "could not determine whether file is a movie or series, skipping"
        );
        return None;
    };

    let Some(title) = parsed_title(parsed) else {
        tracing::warn!(relative_path, "file is missing a parsed title, skipping");
        return None;
    };

    let lower_title = title.to_lowercase();
    let kind_str = format!("{:?}", root_kind);
    let root_id = hash_id("r", &[&first_dir_past_root_dir, &kind_str, &lower_title]);

    if root_kind == RootKind::Movie {
        let item_id = hash_id("i", &[&root_id, "movie"]);
        return Some(FileRecommendation {
            root_id,
            root_kind,
            root_name: title.to_string(),
            season: None,
            items: vec![(item_id, ItemKind::Movie, None, title.to_string())],
        });
    }

    if episode_numbers.is_empty() {
        tracing::warn!(
            relative_path,
            "file was detected as a series entry but has no parsed episode numbers"
        );
        return None;
    }

    let season = season_number.map(|num| {
        let season_id = hash_id("s", &[&root_id, &format!("season {num}")]);
        (season_id, i64::from(num), format!("Season {num}"))
    });

    let lineage_id = season
        .as_ref()
        .map(|(id, _, _)| id.clone())
        .unwrap_or_else(|| root_id.clone());

    let items = episode_numbers
        .into_iter()
        .map(|episode_number| {
            let item_id = hash_id("i", &[&lineage_id, &format!("episode {episode_number}")]);
            (
                item_id,
                ItemKind::Episode,
                Some(i64::from(episode_number)),
                parsed_episode_name(parsed, episode_number),
            )
        })
        .collect::<Vec<_>>();

    Some(FileRecommendation {
        root_id,
        root_kind,
        root_name: title.to_string(),
        season,
        items,
    })
}

fn item_sort_key(item: &ItemAcc, season_order_lookup: &HashMap<String, i64>) -> (i64, i64, i64, String) {
    match item.kind {
        ItemKind::Movie => (0, -1, -1, item.id.clone()),
        ItemKind::Episode => {
            let season_order = item
                .season_id
                .as_ref()
                .and_then(|season_id| season_order_lookup.get(season_id).copied())
                .unwrap_or(-1);
            (
                1,
                season_order,
                item.episode_number.unwrap_or(i64::MAX),
                item.id.clone(),
            )
        }
    }
}

fn item_file_order_for_size(size_bytes: i64) -> i64 {
    -size_bytes
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

fn hash_id(prefix: &str, parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.to_lowercase().as_bytes());
    }
    let result = hasher.finalize();
    let hex = hex::encode(result);
    format!("{}_{}", prefix, &hex[..16])
}
