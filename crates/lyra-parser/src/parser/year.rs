use crate::pattern;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref YEAR_REGEX: Regex =
        pattern!(r"\b(?i)(?<start>[0-9]{4})(?: ?- ?(?<end>[0-9]{2,4}))?\b");
}

pub fn parse_year(input: &str, context: &mut crate::ParserContext) -> (Option<u32>, Option<u32>) {
    // Find all matches, but only keep the last one
    // This avoids "2012 (2009)" being parsed as 2012 when 2009 is the actual release year
    match YEAR_REGEX.captures_iter(input).last() {
        Some(capture) => {
            let full_match = capture.get(0).unwrap();
            context.add_match(full_match.range());

            let start = capture
                .name("start")
                .and_then(|s| s.as_str().parse::<u32>().ok());

            let end_year = capture.name("end").and_then(|end| {
                let end_str = end.as_str();
                let parsed = if end_str.len() < 4 {
                    // If the end year is 2 digits, we assume it's in the same century or next century
                    if let Some(start_year) = start {
                        let start_century = start_year / 100 * 100;
                        let end_short = end_str.parse::<u32>().unwrap();
                        let end_full = start_century + end_short;

                        // If the resulting end year is before the start year,
                        // we assume it's in the next century
                        if end_full < start_year {
                            Some(end_full + 100)
                        } else {
                            Some(end_full)
                        }
                    } else {
                        None
                    }
                } else {
                    end_str.parse::<u32>().ok()
                };
                parsed
            });

            (start, end_year)
        }
        None => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_year() {
        // Test single years
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_year("Movie.2018.1080p", &mut context),
            (Some(2018), None)
        );

        // Test year ranges with abbreviated end year
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_year("Series.2018-19.Complete", &mut context),
            (Some(2018), Some(2019))
        );

        // Test year ranges with full end year
        let mut context = crate::ParserContext::new();
        assert_eq!(
            parse_year("Documentary.2010-2015.BluRay", &mut context),
            (Some(2010), Some(2015))
        );

        // Test no years
        let mut context = crate::ParserContext::new();
        assert_eq!(parse_year("Movie.Without.Year", &mut context), (None, None));

        // Always pick the last year
        let mut context = crate::ParserContext::new();
        assert_eq!(parse_year("2012 (2009)", &mut context), (Some(2009), None));
    }
}
