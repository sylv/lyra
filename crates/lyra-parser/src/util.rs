const SPREAD_WORDS: [&str; 4] = ["-", "to", "..", "~"];

fn split_on_numbers(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut segment_start = 0;
    let first_char = s.chars().next().unwrap();
    let mut in_digit_section = first_char.is_ascii_digit();

    for (i, c) in s.char_indices() {
        let is_digit = c.is_ascii_digit();
        if is_digit != in_digit_section {
            result.push(&s[segment_start..i]);
            segment_start = i;
            in_digit_section = is_digit;
        }

        // stops "S01-S07" from being split into ["S", "01", "-S", "07"]
        // which breaks the parser because "-S" is not a valid spread word
        if SPREAD_WORDS.contains(&&s[segment_start..i]) {
            result.push(&s[segment_start..i]);
            segment_start = i;
        }
    }

    result.push(&s[segment_start..]);
    result
}

/// Parses a string to extract possible ranges of numbers.
/// "Season 1-3" or "Season 1 to 3" will return [1, 2, 3].
/// "Season 1 3" will return [1, 3].
/// "Season 2" will return [2].
pub fn parse_possible_range(input: &str, force_range: bool) -> Vec<u32> {
    let split = split_on_numbers(input);

    let mut spread = false;
    let mut numbers = Vec::new();
    for word in split {
        let trimmed = word.trim();
        if !force_range && SPREAD_WORDS.contains(&trimmed) {
            spread = true;
            continue;
        } else if let Ok(num) = word.parse::<u32>() {
            numbers.push(num);
        } else {
            // Check if the word contains a spread word (like " - E" containing "-")
            if !force_range
                && SPREAD_WORDS
                    .iter()
                    .any(|&spread_word| trimmed.contains(spread_word))
            {
                spread = true;
            }
        }
    }

    if (spread || force_range) && numbers.len() == 2 {
        let start = numbers[0];
        let end = numbers[1];
        return (start..=end).collect();
    }

    return numbers;
}

// /// Parses a string to extract a single number.
// /// "Season 1" will return Some(1).
// /// "Season 1-3" will return None because two numbers are found.
// pub fn parse_number(input: &str) -> Option<u32> {
//     let split = split_on_numbers(input);
//     let mut numbers = Vec::new();
//     for word in split {
//         if let Ok(num) = word.parse::<u32>() {
//             numbers.push(num);
//         }
//     }

//     if numbers.len() == 1 {
//         return Some(numbers[0]);
//     }

//     None
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_possible_range("Season 1", false), vec![1]);
        assert_eq!(parse_possible_range("Season 1-3", false), vec![1, 2, 3]);
        assert_eq!(parse_possible_range("Season 1 - 3", false), vec![1, 2, 3]);
        assert_eq!(parse_possible_range("Season 1 to 3", false), vec![1, 2, 3]);
        assert_eq!(parse_possible_range("Season 1 3 6", false), vec![1, 3, 6]);
        assert_eq!(parse_possible_range("Season 1,3,6", false), vec![1, 3, 6]);
        assert_eq!(parse_possible_range("Season 1 3", false), vec![1, 3]);
        assert_eq!(
            parse_possible_range("E10 - E17", false),
            vec![10, 11, 12, 13, 14, 15, 16, 17]
        );
        assert_eq!(
            parse_possible_range("S01-S07", false),
            vec![1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(parse_possible_range("E01E03", false), vec![1, 3]);
        assert_eq!(parse_possible_range("E01E03", true), vec![1, 2, 3]);
        assert_eq!(
            parse_possible_range("S01-S07", true),
            vec![1, 2, 3, 4, 5, 6, 7]
        );
    }

    #[test]
    fn test_split_on_numbers() {
        assert_eq!(
            split_on_numbers("Season 1-3"),
            vec!["Season ", "1", "-", "3"]
        );
        assert_eq!(
            split_on_numbers("Season 1 to 3"),
            vec!["Season ", "1", " to ", "3"]
        );
        assert_eq!(
            split_on_numbers("Season 1 3"),
            vec!["Season ", "1", " ", "3"]
        );
        assert_eq!(
            split_on_numbers("Season 1,3,6"),
            vec!["Season ", "1", ",", "3", ",", "6"]
        );
        assert_eq!(split_on_numbers("S01-S07"), vec!["S", "01", "-", "S", "07"])
    }
}
