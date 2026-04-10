use lyra_probe::VideoKeyframes;
use std::time::Duration;

/// Compute segment lengths in PTS units by cutting at keyframes at/after each desired cut point.
/// keyframes_pts must be sorted ascending.
pub fn compute_segments_from_keyframes_pts(
    keyframes_pts: &[i64],
    total_duration_pts: i64,
    desired_segment_length_pts: i64,
) -> Result<Vec<i64>, String> {
    if !keyframes_pts.is_empty() && total_duration_pts < *keyframes_pts.last().unwrap() {
        return Err("Invalid duration in keyframe data".to_string());
    }
    if desired_segment_length_pts <= 0 || total_duration_pts <= 0 {
        return Err("Invalid segment length or duration".to_string());
    }

    let mut last_keyframe: i64 = 0;
    let mut desired_cut_time = desired_segment_length_pts;
    let mut segments = Vec::new();

    for &kf in keyframes_pts {
        if kf >= desired_cut_time {
            let segment_len_pts = kf - last_keyframe;
            segments.push(segment_len_pts);
            last_keyframe = kf;
            desired_cut_time += desired_segment_length_pts;
        }
    }

    let tail_pts = total_duration_pts - last_keyframe;
    segments.push(tail_pts);
    Ok(segments)
}

pub fn create_fmp4_hls_playlist_from_segment_starts_pts(
    segment_start_pts: &[i64],
    total_duration_pts: i64,
    time_base_num: i64,
    time_base_den: i64,
    endpoint_prefix: &str,
    endpoint_suffix: &str,
) -> Result<String, String> {
    if segment_start_pts.is_empty() {
        return Err("segment_start_pts cannot be empty".to_string());
    }
    if total_duration_pts <= 0 {
        return Err("total_duration_pts must be positive".to_string());
    }

    let mut playlist = String::new();
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:7\n");
    playlist.push_str("#EXT-X-TARGETDURATION:7\n");
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
    playlist.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");
    playlist.push_str(&format!(
        "#EXT-X-MAP:URI=\"{}init.mp4{}\"\n",
        endpoint_prefix, endpoint_suffix
    ));

    for (index, &start_pts) in segment_start_pts.iter().enumerate() {
        let end_pts = segment_start_pts
            .get(index + 1)
            .copied()
            .unwrap_or(total_duration_pts);
        if end_pts < start_pts {
            return Err("segment_start_pts must be non-decreasing".to_string());
        }
        let seg_pts = end_pts - start_pts;
        let dur_seconds = (seg_pts as f64) * (time_base_num as f64) / (time_base_den as f64);
        playlist.push_str(&format!("#EXTINF:{:.6},\n", dur_seconds));
        playlist.push_str(&format!(
            "{}{}.m4s{}?startPts={}\n",
            endpoint_prefix, index, endpoint_suffix, start_pts
        ));
    }

    playlist.push_str("#EXT-X-ENDLIST\n");
    Ok(playlist)
}

pub fn seconds_to_pts(seconds: f64, time_base_num: i64, time_base_den: i64) -> i64 {
    let pts = seconds * (time_base_den as f64) / (time_base_num as f64);
    pts.round() as i64
}

pub fn create_hls_cuts(
    keyframes: &VideoKeyframes,
    desired_segment_duration: Duration,
) -> Option<String> {
    let cuts = keyframes
        .segment_start_pts(desired_segment_duration)
        .into_iter()
        .skip(1)
        .map(|pts| keyframes.pts_to_micros(pts).to_string())
        .collect::<Vec<_>>();

    if cuts.is_empty() {
        None
    } else {
        Some(cuts.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::create_hls_cuts;
    use lyra_probe::VideoKeyframes;
    use std::time::Duration;

    #[test]
    fn hls_cuts_are_derived_in_packager() {
        let keyframes = VideoKeyframes::new(0, 1, 1, vec![0, 3, 6, 9, 12, 15]).unwrap();

        assert_eq!(
            create_hls_cuts(&keyframes, Duration::from_secs(6)).as_deref(),
            Some("6000000,12000000")
        );
    }
}
