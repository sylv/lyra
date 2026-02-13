use crate::pattern;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // IMDb ID pattern: tt followed by 7-12 digits
    static ref IMDB_PATTERN: Regex = pattern!(r"(?i)\btt\d{7,12}\b");

    // Provider ID patterns: provider optionally followed by id, then dash/equals, then 1-12 digits
    static ref TMDB_PATTERN: Regex = pattern!(r"(?i)\btmdb(?:id)?\s*[-=]\s*(\d{1,12})\b");
    static ref TVDB_PATTERN: Regex = pattern!(r"(?i)\btvdb(?:id)?\s*[-=]\s*(\d{1,12})\b");
    static ref ANIDB_PATTERN: Regex = pattern!(r"(?i)\banidb(?:id)?\s*[-=]\s*(\d{1,12})\b");
    static ref TRAKT_PATTERN: Regex = pattern!(r"(?i)\btrakt(?:id)?\s*[-=]\s*(\d{1,12})\b");
}

/// Parse IMDb, TMDb, TVDb, AniDB, and Trakt IDs from input string.
pub fn parse_ids(
    input: &str,
    context: &mut crate::ParserContext,
) -> (
    Option<String>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
) {
    let imdb_id = IMDB_PATTERN.find(input).map(|m| {
        context.add_match(m.range());
        m.as_str().to_ascii_lowercase()
    });

    let tmdb_id = parse_numeric_id(input, &TMDB_PATTERN, context);
    let tvdb_id = parse_numeric_id(input, &TVDB_PATTERN, context);
    let anidb_id = parse_numeric_id(input, &ANIDB_PATTERN, context);
    let trakt_id = parse_numeric_id(input, &TRAKT_PATTERN, context);

    (imdb_id, tmdb_id, tvdb_id, anidb_id, trakt_id)
}

fn parse_numeric_id(
    input: &str,
    pattern: &Regex,
    context: &mut crate::ParserContext,
) -> Option<u64> {
    pattern.captures(input).and_then(|caps| {
        let full_match = caps.get(0)?;
        context.add_match(full_match.range());
        caps.get(1)?.as_str().parse().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imdb_id_parsing() {
        // Valid IMDb IDs
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie Name tt1234567 (2016)", &mut context),
            (Some("tt1234567".to_string()), None, None, None, None)
        );

        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Another Film tt12345678901 1080p", &mut context),
            (Some("tt12345678901".to_string()), None, None, None, None)
        );

        // Minimum length (7 digits)
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Short ID tt1234567", &mut context),
            (Some("tt1234567".to_string()), None, None, None, None)
        );

        // Maximum length (12 digits)
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Long ID tt123456789012", &mut context),
            (Some("tt123456789012".to_string()), None, None, None, None)
        );
    }

    #[test]
    fn test_tmdb_id_parsing() {
        // Various TMDb formats
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie tmdb-12345 (2016)", &mut context),
            (None, Some(12345), None, None, None)
        );

        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Film tmdbid=67890 1080p", &mut context),
            (None, Some(67890), None, None, None)
        );

        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Title tmdb-123456789012", &mut context),
            (None, Some(123456789012), None, None, None)
        );

        // Case insensitive
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie TMDB-12345", &mut context),
            (None, Some(12345), None, None, None)
        );

        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Film TMDbID=67890", &mut context),
            (None, Some(67890), None, None, None)
        );
    }

    #[test]
    fn test_tvdb_anidb_trakt_parsing() {
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Show tvdb-101112 anidb=4455 traktid-9988", &mut context),
            (None, None, Some(101112), Some(4455), Some(9988))
        );
    }

    #[test]
    fn test_both_ids_present() {
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie tt1234567 tmdb-12345 (2016)", &mut context),
            (Some("tt1234567".to_string()), Some(12345), None, None, None)
        );

        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Film tmdbid=67890 tt9876543210 1080p", &mut context),
            (
                Some("tt9876543210".to_string()),
                Some(67890),
                None,
                None,
                None
            )
        );
    }

    #[test]
    fn test_invalid_formats() {
        // Too short IMDb ID (less than 7 digits)
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie tt123456", &mut context),
            (None, None, None, None, None)
        );

        // Too long IMDb ID (more than 12 digits)
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie tt1234567890123", &mut context),
            (None, None, None, None, None)
        );

        // Invalid IMDb format (no tt prefix)
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie 1234567", &mut context),
            (None, None, None, None, None)
        );

        // Invalid TMDb format (no separator)
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie tmdb12345", &mut context),
            (None, None, None, None, None)
        );

        // TMDb ID too long (more than 12 digits)
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie tmdb-1234567890123", &mut context),
            (None, None, None, None, None)
        );
    }

    #[test]
    fn test_no_ids() {
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Regular Movie (2016) 1080p", &mut context),
            (None, None, None, None, None)
        );
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Film without any IDs", &mut context),
            (None, None, None, None, None)
        );
    }

    #[test]
    fn test_multiple_matches() {
        // Should return the first match for each type
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids(
                "Movie tt1234567 and tt7654321 tmdb-111 tmdb=222 tvdb=3 tvdb-4",
                &mut context
            ),
            (
                Some("tt1234567".to_string()),
                Some(111),
                Some(3),
                None,
                None
            )
        );
    }

    #[test]
    fn test_word_boundaries() {
        // Should not match partial matches
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie nottt1234567", &mut context),
            (None, None, None, None, None)
        );
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie tt1234567suffix", &mut context),
            (None, None, None, None, None)
        );
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_ids("Movie sometmdb-12345", &mut context),
            (None, None, None, None, None)
        );
    }
}
