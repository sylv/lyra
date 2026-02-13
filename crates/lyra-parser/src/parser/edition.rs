use crate::pattern;
use lazy_static::lazy_static;
use regex::Regex;

fn strip(name: &str) -> String {
    lazy_static! {
        // "director s cut" -> "director's cut"
        static ref APOST_FIX_REGEX: Regex = Regex::new(r"(?i)([a-z]{2,}) (s)\b").unwrap();
        static ref STRIP_REGEX: Regex = Regex::new(r"[ ._!\?']+").unwrap();
    }

    let stripped = STRIP_REGEX
        .replace_all(name, " ")
        .trim()
        .to_lowercase()
        .to_string();

    return APOST_FIX_REGEX.replace_all(&stripped, "$1's").to_string();
}

fn normalize_edition(text: &str) -> String {
    // Replace dots with spaces and clean up extra whitespace
    let normalized = text.replace('.', " ");
    let normalized = regex::Regex::new(r"\s+")
        .unwrap()
        .replace_all(&normalized, " ");
    let normalized = normalized.trim();

    // Apply apostrophe fix to the entire string before splitting
    let normalized = strip(&normalized);

    // Apply proper capitalization
    let words: Vec<&str> = normalized.split_whitespace().collect();
    let capitalized: Vec<String> = words
        .iter()
        .map(|word| match word.to_lowercase().as_str() {
            "cut" => "Cut".to_string(),
            "edition" => "Edition".to_string(),
            "director's" => "Director's".to_string(),
            "directors" => "Director's".to_string(),
            "collector's" => "Collector's".to_string(),
            "collectors" => "Collector's".to_string(),
            _ => capitalize_first(word),
        })
        .collect();

    capitalized.join(" ")
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first
            .to_uppercase()
            .chain(chars.as_str().to_lowercase().chars())
            .collect(),
    }
}

lazy_static! {
    // Plex format patterns - these are prioritized as they're more specific
    static ref PLEX_PATTERN: Regex = pattern!(r"(?i)(?:\{|\[) ?edition ?(?:-|=) ?(?<edition>[^}]+) ?(?:\}|\])");

    // Free-form patterns - less specific, used as fallback
    static ref FREEFORM_PATTERNS: Vec<Regex> = vec![
        pattern!(r"(?i)\b(Director'?s?[.\s]*Cut)\b"),
        pattern!(r"(?i)\b(Extended[.\s]*(?:Cut|Edition))\b"),
        pattern!(r"(?i)\b(Theatrical[.\s]*(?:Cut|Edition))\b"),
        pattern!(r"(?i)\b(Final[.\s]*Cut)\b"),
        pattern!(r"(?i)\b(Ultimate[.\s]*Cut)\b"),
        pattern!(r"(?i)\b(International[.\s]*Cut)\b"),
        pattern!(r"(?i)\b(Uncut)\b"),
        pattern!(r"(?i)\b(Unrated)\b"),
        pattern!(r"(?i)\b(Uncensored)\b"),
        pattern!(r"(?i)\b(Remastered|4K[.\s]*Remaster)\b"),
        pattern!(r"(?i)\b(Upscaled|AI[.\s]*Upscaled)\b"),
        pattern!(r"(?i)\b(Redux)\b"),
        pattern!(r"(?i)\b(Special[.\s]*Edition)\b"),
        pattern!(r"(?i)\b(Collector'?s?[.\s]*Edition)\b"),
        pattern!(r"(?i)\b((?:\d+th[.\s]*)?Anniversary[.\s]*Edition)\b"),
        pattern!(r"(?i)\b(Complete[.\s]*Edition)\b"),
        pattern!(r"(?i)\b(Definitive[.\s]*Edition)\b"),
        pattern!(r"(?i)\b(Remaster[.\s]*Edition)\b"),
        pattern!(r"(?i)\b(Limited[.\s]*Edition)\b"),
    ];
}

pub fn parse_edition(input: &str, context: &mut crate::ParserContext) -> Option<String> {
    // First, try to find Plex format matches (higher priority)
    if let Some(capture) = PLEX_PATTERN.captures(input) {
        let edition_text = capture.name("edition").unwrap();
        context.add_match(edition_text.range());
        return Some(normalize_edition(edition_text.as_str()));
    }

    // Fall back to free-form patterns
    let mut matches: Vec<(usize, String)> = Vec::new();

    for regex in FREEFORM_PATTERNS.iter() {
        for capture in regex.find_iter(input) {
            context.add_match(capture.range());
            matches.push((capture.start(), normalize_edition(capture.as_str())));
        }
    }

    // Sort matches by position (descending to prefer matches at the end)
    matches.sort_by(|a, b| b.0.cmp(&a.0));

    // Return the first match (the one closest to the end)
    matches.first().map(|(_, edition)| edition.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plex_format_parsing() {
        let mut c = crate::ParserContext::new();
        assert_eq!(
            parse_edition("Movie Name {edition-Director's Cut}", &mut c),
            Some("Director's Cut".to_string())
        );

        assert_eq!(
            parse_edition("Another Movie {edition-Extended Edition}", &mut c),
            Some("Extended Edition".to_string())
        );

        assert_eq!(
            parse_edition("Test Film {edition-Uncut}", &mut c),
            Some("Uncut".to_string())
        );

        // Test dots in plex format
        assert_eq!(
            parse_edition("Movie.[edition=Director.s.Cut]", &mut c),
            Some("Director's Cut".to_string())
        );
    }

    #[test]
    fn test_freeform_parsing() {
        let mut c = crate::ParserContext::new();
        assert_eq!(
            parse_edition("Movie Name (2016) Final Cut", &mut c),
            Some("Final Cut".to_string())
        );

        assert_eq!(
            parse_edition("Some Film Uncensored 1080p", &mut c),
            Some("Uncensored".to_string())
        );

        assert_eq!(
            parse_edition("Title Upscaled BluRay", &mut c),
            Some("Upscaled".to_string())
        );
    }

    #[test]
    fn test_dots_as_spaces() {
        let mut c = crate::ParserContext::new();
        assert_eq!(
            parse_edition("Movie.Name.Directors.Cut.2016", &mut c),
            Some("Director's Cut".to_string())
        );

        assert_eq!(
            parse_edition("Film.Extended.Edition.1080p", &mut c),
            Some("Extended Edition".to_string())
        );

        assert_eq!(
            parse_edition("Title.Final.Cut.BluRay", &mut c),
            Some("Final Cut".to_string())
        );
    }

    #[test]
    fn test_plex_preference() {
        let mut c = crate::ParserContext::new();
        // Should prefer Plex format over free-form when both are present
        assert_eq!(
            parse_edition("Movie Final Cut {edition-Director's Cut}", &mut c),
            Some("Director's Cut".to_string())
        );
    }

    #[test]
    fn test_normalization() {
        let mut c = crate::ParserContext::new();
        // Test proper capitalization
        assert_eq!(
            parse_edition("movie directors cut", &mut c),
            Some("Director's Cut".to_string())
        );

        assert_eq!(
            parse_edition("EXTENDED EDITION", &mut c),
            Some("Extended Edition".to_string())
        );

        assert_eq!(
            parse_edition("collectors edition", &mut c),
            Some("Collector's Edition".to_string())
        );
    }

    #[test]
    fn test_no_match() {
        let mut c = crate::ParserContext::new();
        assert_eq!(parse_edition("Regular Movie 2016 1080p", &mut c), None);
    }
}
