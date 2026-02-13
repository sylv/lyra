use std::collections::HashMap;
use std::path::Path;

pub fn infer_additional_episode_numbers(file_paths: &[String]) -> Vec<Vec<u32>> {
    let mut results = vec![Vec::new(); file_paths.len()];

    if file_paths.len() < 2 {
        return results;
    }

    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (index, file_path) in file_paths.iter().enumerate() {
        let parent = Path::new(file_path).parent().and_then(|p| {
            let parent_str = p.to_string_lossy();
            if parent_str.is_empty() {
                None
            } else {
                Some(parent_str.into_owned())
            }
        });

        if let Some(parent) = parent {
            groups.entry(parent).or_default().push(index);
        }
    }

    for (parent_path, indices) in groups {
        if indices.len() < 2 {
            continue;
        }

        let prefix_len = match common_prefix_len(indices.as_slice(), file_paths) {
            Some(len) => len,
            None => continue,
        };

        if prefix_len <= parent_path.len() {
            continue;
        }

        let mut extracted = Vec::with_capacity(indices.len());
        let mut valid = true;

        for &idx in &indices {
            let file_path = &file_paths[idx];
            if prefix_len >= file_path.len() {
                valid = false;
                break;
            }

            let rest = &file_path[prefix_len..];
            let digit_count = rest
                .as_bytes()
                .iter()
                .take_while(|byte| byte.is_ascii_digit())
                .count();

            if digit_count == 0 {
                valid = false;
                break;
            }

            let digits = &rest[..digit_count];
            match digits.parse::<u32>() {
                Ok(number) => extracted.push((idx, number)),
                Err(_) => {
                    valid = false;
                    break;
                }
            }
        }

        if !valid {
            continue;
        }

        for (idx, number) in extracted {
            results[idx].push(number);
        }
    }

    results
}

fn common_prefix_len(indices: &[usize], files: &[String]) -> Option<usize> {
    let first_index = *indices.first()?;
    let first = files[first_index].as_str();
    let mut prefix_len = first.len();

    for &idx in indices.iter().skip(1) {
        let current = files[idx].as_str();
        prefix_len = prefix_len.min(shared_prefix_len(first, current));
        if prefix_len == 0 {
            return None;
        }
    }

    Some(prefix_len)
}

fn shared_prefix_len(a: &str, b: &str) -> usize {
    let bytes_a = a.as_bytes();
    let bytes_b = b.as_bytes();
    let mut i = 0;
    let max = bytes_a.len().min(bytes_b.len());

    while i < max && bytes_a[i] == bytes_b[i] {
        i += 1;
    }

    while i > 0 && (!a.is_char_boundary(i) || !b.is_char_boundary(i)) {
        i -= 1;
    }

    i
}
