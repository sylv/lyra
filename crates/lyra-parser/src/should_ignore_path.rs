use lazy_static::lazy_static;
use regex::Regex;

const VIDEO_EXTS: [&str; 9] = [
    ".mkv", ".mp4", ".avi", ".mov", ".wmv", ".flv", ".webm", ".mpeg", ".mpg",
];

lazy_static! {
    static ref PATH_FILTERS: Vec<Regex> = vec![
        Regex::new(r"^lore$").unwrap(),
        Regex::new(r"^histories(( and| &) lore)?$").unwrap(),
        Regex::new(r"sample").unwrap(),
        Regex::new(r"^behind.the.scenes$").unwrap(),
        Regex::new(r"^deleted.and.extended.scenes$").unwrap(),
        Regex::new(r"^deleted.scenes$").unwrap(),
        Regex::new(r"^extras?$").unwrap(),
        Regex::new(r"^featurettes$").unwrap(),
        Regex::new(r"^other$").unwrap(),
        Regex::new(r"^interviews$").unwrap(),
        Regex::new(r"^scenes$").unwrap(),
        Regex::new(r"^shorts$").unwrap(),
        Regex::new(r"^specials?$").unwrap(),
        Regex::new(r"^trailers?$").unwrap(),
        Regex::new(r"^soundtracks?$").unwrap()
    ];
}

pub fn is_video_file(input: &str) -> bool {
    let haystack = input.to_lowercase();
    VIDEO_EXTS.iter().any(|ext| haystack.ends_with(ext))
}

pub fn should_ignore_path(input: &str) -> bool {
    if !is_video_file(input) {
        return true;
    }

    return path_is_ignored(input);
}

pub fn path_is_ignored(input: &str) -> bool {
    for path_part in input.split('/') {
        if path_part.is_empty() {
            continue;
        }

        let path_part = path_part.to_lowercase();
        let is_filtered = PATH_FILTERS.iter().any(|regex| regex.is_match(&path_part));
        if is_filtered {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore_path() {
        assert!(should_ignore_path("files/samples/video.mp4"));
        assert!(!should_ignore_path(
            "trailer park boys/season 1/episode 1.mk4"
        ));
    }
}
