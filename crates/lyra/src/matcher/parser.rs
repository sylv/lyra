use crate::matcher::matcher::ParsedFile;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // torrent_name_parsers year regex is too loose, eg "EDGE2020" is parsed to 2020
    // when it shouldnt be
    static ref YEAR_REGEX: Regex = Regex::new(r"\b\d{4}\b").unwrap();
    // unlike the tmdb id, imdb ids start with "tt" so we can match them much more loosely
    // this might cause issues but for now im comfortable doing this
    static ref IMDB_ID_REGEX: Regex = Regex::new(r"\btt\d{6,}\b").unwrap();
    // this is based off the plex patterns
    // https://regex101.com/r/95ms08/1
    static ref TMDB_ID_REGEX: Regex = Regex::new(r"(?:\{|\[)tmdb(?:-?id)?(?:=|-)(?<id>[0-9]{3,})(?:\}|\])").unwrap();
}

pub fn get_tmdb_id(file_name: &str) -> Option<i64> {
    TMDB_ID_REGEX
        .captures(file_name)
        .map(|c| c.name("id").unwrap().as_str().parse::<i64>().unwrap())
}

pub fn get_imdb_id(file_name: &str) -> Option<String> {
    IMDB_ID_REGEX
        .find(file_name)
        .map(|m| m.as_str().to_string())
}

pub fn parse_file_name(file_name: &str) -> Result<ParsedFile> {
    let metadata = torrent_name_parser::Metadata::from(&file_name)?;

    // for year, we have to get the last occurence or else a movie like "2077 (2009)" would
    // be parsed as "2077" when it was released in 2009.
    let year = YEAR_REGEX
        .find_iter(file_name)
        .last()
        .map(|m| m.as_str().parse::<i32>().unwrap());

    let season = metadata.season();
    let episodes = metadata.episodes();

    Ok(ParsedFile {
        title: metadata.title().to_string(),
        year,
        season_number: season,
        episodes: episodes.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_name_1() {
        let file_name = "The.Matrix.1999.1080p.BluRay.x264.mkv";
        let matchable = parse_file_name(file_name).unwrap();
        assert_eq!(matchable.title, "The Matrix");
        assert_eq!(matchable.year, Some(1999));

        let tmdb_id = get_tmdb_id(file_name);
        assert_eq!(tmdb_id, None);
    }

    #[test]
    fn test_parse_file_name_2() {
        let file_name = "Batman Begins (2005) {imdb-tt0372784}.mp4";
        let matchable = parse_file_name(file_name).unwrap();
        assert_eq!(matchable.title, "Batman Begins");
        assert_eq!(matchable.year, Some(2005));

        let imdb_id = get_imdb_id(file_name);
        assert_eq!(imdb_id, Some("tt0372784".to_string()));
    }

    #[test]
    fn test_parse_file_name_3() {
        let file_name = "Arcane (2021) {imdb-tt11126994}/Season 1/Arcane (2021) - S01E01 - Welcome to the Playground [NF WEBDL-1080p ReleaseGroup]";
        let matchable = parse_file_name(file_name).unwrap();
        assert_eq!(matchable.title, "Arcane");
        assert_eq!(matchable.year, Some(2021));

        let imdb_id = get_imdb_id(file_name);
        assert_eq!(imdb_id, Some("tt11126994".to_string()));
    }
}
