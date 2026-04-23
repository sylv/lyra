#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageMatch {
    pub canonical: String,
    pub primary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LanguageMatchStrength {
    Primary = 1,
    Exact = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubtitleTrackVariant {
    Forced,
    Normal,
    Sdh,
    Commentary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubtitleSelectionCandidate {
    pub id: String,
    pub language_bcp47: Option<String>,
    pub variant: SubtitleTrackVariant,
}

pub fn canonicalize_language_tag(tag: &str) -> Option<LanguageMatch> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parts = trimmed
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let primary = parts.first()?.to_ascii_lowercase();
    if !primary.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }

    let canonical = parts
        .into_iter()
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                part.to_ascii_lowercase()
            } else if part.len() == 2 && part.chars().all(|ch| ch.is_ascii_alphabetic()) {
                part.to_ascii_uppercase()
            } else {
                part.to_ascii_lowercase()
            }
        })
        .collect::<Vec<_>>()
        .join("-");

    Some(LanguageMatch { canonical, primary })
}

pub fn language_match_strength(candidate: &str, wanted: &str) -> Option<LanguageMatchStrength> {
    let candidate = canonicalize_language_tag(candidate)?;
    let wanted = canonicalize_language_tag(wanted)?;
    if candidate.canonical == wanted.canonical {
        return Some(LanguageMatchStrength::Exact);
    }
    if candidate.primary == wanted.primary {
        return Some(LanguageMatchStrength::Primary);
    }
    None
}

pub fn dedupe_language_hints(preferred: &[String], hints: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    for tag in preferred.iter().chain(hints.iter()) {
        let Some(canonical) = canonicalize_language_tag(tag) else {
            continue;
        };
        if merged.iter().any(|existing: &String| {
            canonicalize_language_tag(existing)
                .is_some_and(|existing| existing.canonical == canonical.canonical)
        }) {
            continue;
        }
        merged.push(canonical.canonical);
    }
    merged
}

pub fn move_language_to_front(existing_json: &str, selected: Option<&str>) -> String {
    let mut languages: Vec<String> = serde_json::from_str(existing_json).unwrap_or_default();
    let Some(selected) = selected.and_then(canonicalize_language_tag) else {
        return serde_json::to_string(&languages).unwrap_or_else(|_| "[]".to_string());
    };

    languages.retain(|lang| {
        canonicalize_language_tag(lang)
            .is_none_or(|lang| lang.canonical != selected.canonical)
    });
    languages.insert(0, selected.canonical);
    serde_json::to_string(&languages).unwrap_or_else(|_| "[]".to_string())
}

pub fn select_subtitle_track(
    tracks: &[SubtitleSelectionCandidate],
    subtitle_mode: SubtitleMode,
    preferred_languages: &[String],
    language_hints: &[String],
    subtitle_variant_preference: SubtitleVariantPreference,
    active_audio_language: Option<&str>,
) -> Option<String> {
    if subtitle_mode == SubtitleMode::Off {
        return None;
    }

    let merged_hints = dedupe_language_hints(preferred_languages, language_hints);
    let active_audio_language = active_audio_language
        .and_then(canonicalize_language_tag)
        .map(|lang| lang.canonical);

    let mut best: Option<((bool, bool, i32, i32, i32), &SubtitleSelectionCandidate)> = None;
    for track in tracks {
        if subtitle_mode == SubtitleMode::ForcedOnly && track.variant != SubtitleTrackVariant::Forced {
            continue;
        }

        if subtitle_mode == SubtitleMode::On
            && track.variant == SubtitleTrackVariant::Commentary
            && subtitle_variant_preference != SubtitleVariantPreference::Commentary
        {
            continue;
        }

        if subtitle_variant_preference != SubtitleVariantPreference::Auto
            && !variant_matches_preference(track.variant, subtitle_variant_preference)
        {
            continue;
        }

        let language = track.language_bcp47.as_deref()?;
        let audio_match = active_audio_language
            .as_deref()
            .and_then(|audio| language_match_strength(language, audio));
        let ordered_match = merged_hints
            .iter()
            .enumerate()
            .filter_map(|(index, hint)| {
                language_match_strength(language, hint).map(|strength| (index, strength))
            })
            .max_by_key(|(_, strength)| *strength)
            .map(|(index, strength)| (index as i32, strength as i32));

        match subtitle_mode {
            SubtitleMode::ForcedOnly => {
                if track.variant != SubtitleTrackVariant::Forced
                    || audio_match.is_none()
                        && ordered_match.is_none()
                {
                    continue;
                }
            }
            SubtitleMode::On => {
                if ordered_match.is_none() {
                    continue;
                }
            }
            SubtitleMode::Off => continue,
        }

        let language_order = ordered_match.map(|(index, _)| -index).unwrap_or(-10_000);
        let language_strength = ordered_match
            .map(|(_, strength)| strength)
            .or_else(|| audio_match.map(|strength| strength as i32))
            .unwrap_or_default();
        let forced_audio_bonus = (track.variant == SubtitleTrackVariant::Forced
            && audio_match.is_some()
            && subtitle_variant_preference == SubtitleVariantPreference::Auto) as i32;
        let variant_rank = variant_rank(track.variant, subtitle_variant_preference);
        let score = (
            forced_audio_bonus > 0,
            audio_match == Some(LanguageMatchStrength::Exact),
            variant_rank,
            language_strength,
            language_order,
        );

        if best.as_ref().is_none_or(|(best_score, _)| score > *best_score) {
            best = Some((score, track));
        }
    }

    best.map(|(_, track)| track.id.clone())
}

fn variant_matches_preference(
    variant: SubtitleTrackVariant,
    preference: SubtitleVariantPreference,
) -> bool {
    matches!(
        (variant, preference),
        (SubtitleTrackVariant::Forced, SubtitleVariantPreference::Forced)
            | (SubtitleTrackVariant::Normal, SubtitleVariantPreference::Normal)
            | (SubtitleTrackVariant::Sdh, SubtitleVariantPreference::Sdh)
            | (SubtitleTrackVariant::Commentary, SubtitleVariantPreference::Commentary)
    )
}

fn variant_rank(variant: SubtitleTrackVariant, preference: SubtitleVariantPreference) -> i32 {
    match preference {
        SubtitleVariantPreference::Auto => match variant {
            SubtitleTrackVariant::Normal => 4,
            SubtitleTrackVariant::Sdh => 3,
            SubtitleTrackVariant::Forced => 2,
            SubtitleTrackVariant::Commentary => 1,
        },
        SubtitleVariantPreference::Forced if variant == SubtitleTrackVariant::Forced => 10,
        SubtitleVariantPreference::Normal if variant == SubtitleTrackVariant::Normal => 10,
        SubtitleVariantPreference::Sdh if variant == SubtitleTrackVariant::Sdh => 10,
        SubtitleVariantPreference::Commentary if variant == SubtitleTrackVariant::Commentary => 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str, language: &str, variant: SubtitleTrackVariant) -> SubtitleSelectionCandidate {
        SubtitleSelectionCandidate {
            id: id.to_string(),
            language_bcp47: Some(language.to_string()),
            variant,
        }
    }

    #[test]
    fn language_matching_handles_primary_and_exact() {
        assert_eq!(
            language_match_strength("en-AU", "en"),
            Some(LanguageMatchStrength::Primary)
        );
        assert_eq!(
            language_match_strength("en-AU", "en-AU"),
            Some(LanguageMatchStrength::Exact)
        );
        assert_eq!(
            language_match_strength("en-AU", "en-US"),
            Some(LanguageMatchStrength::Primary)
        );
    }

    #[test]
    fn exact_match_outranks_primary_only() {
        let picked = select_subtitle_track(
            &[
                candidate("primary", "en-GB", SubtitleTrackVariant::Normal),
                candidate("exact", "en-AU", SubtitleTrackVariant::Normal),
            ],
            SubtitleMode::On,
            &[],
            &["en-AU".to_string()],
            SubtitleVariantPreference::Auto,
            None,
        );
        assert_eq!(picked.as_deref(), Some("exact"));
    }

    #[test]
    fn browser_hints_only_affect_ordering() {
        let merged = dedupe_language_hints(&["fr".to_string()], &["en-AU".to_string(), "fr".to_string()]);
        assert_eq!(merged, vec!["fr".to_string(), "en-AU".to_string()]);
    }

    #[test]
    fn forced_only_uses_audio_language() {
        let picked = select_subtitle_track(
            &[
                candidate("forced-en", "en", SubtitleTrackVariant::Forced),
                candidate("forced-fr", "fr", SubtitleTrackVariant::Forced),
            ],
            SubtitleMode::ForcedOnly,
            &[],
            &[],
            SubtitleVariantPreference::Auto,
            Some("en-US"),
        );
        assert_eq!(picked.as_deref(), Some("forced-en"));
    }

    #[test]
    fn commentary_never_autoselects_in_auto_mode() {
        let picked = select_subtitle_track(
            &[
                candidate("commentary", "en", SubtitleTrackVariant::Commentary),
                candidate("normal", "en", SubtitleTrackVariant::Normal),
            ],
            SubtitleMode::On,
            &["en".to_string()],
            &[],
            SubtitleVariantPreference::Auto,
            None,
        );
        assert_eq!(picked.as_deref(), Some("normal"));
    }

    #[test]
    fn ordered_preferred_languages_beat_later_browser_hints() {
        let picked = select_subtitle_track(
            &[
                candidate("fr", "fr", SubtitleTrackVariant::Normal),
                candidate("en", "en", SubtitleTrackVariant::Normal),
            ],
            SubtitleMode::On,
            &["fr".to_string()],
            &["en".to_string()],
            SubtitleVariantPreference::Auto,
            None,
        );
        assert_eq!(picked.as_deref(), Some("fr"));
    }
}
use crate::entities::users::{SubtitleMode, SubtitleVariantPreference};
