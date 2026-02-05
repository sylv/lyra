pub fn seconds_to_pts(seconds: f64, time_base_num: i64, time_base_den: i64) -> i64 {
    // pts = seconds / (time_base_num / time_base_den) = seconds * time_base_den / time_base_num
    let pts = seconds * (time_base_den as f64) / (time_base_num as f64);
    round_to_even(pts)
}

fn round_to_even(value: f64) -> i64 {
    // Banker's rounding (ties to even) to match Jellyfin PTS handling.
    let floor = value.floor();
    let frac = value - floor;
    if (frac - 0.5).abs() < 1e-9 {
        let base = floor as i64;
        if base % 2 == 0 { base } else { base + 1 }
    } else {
        value.round() as i64
    }
}

fn append_query_param(base: &str, key: &str, value: i64) -> String {
    if base.is_empty() {
        format!("?{}={}", key, value)
    } else if base.contains('?') {
        format!("{}&{}={}", base, key, value)
    } else {
        format!("?{}={}", key, value)
    }
}

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

/// Create an fMP4 HLS VOD playlist using PTS-based keyframe cuts.
///
/// - `time_base_num/den` are the stream time base (e.g., 1/90000).
/// - `endpoint_prefix` is like "hls1/main/".
/// - `query_string` should include any existing query (e.g., "?SegmentContainer=mp4"),
///   and this function will append `startPts=<pts>` for each segment.
pub fn create_fmp4_hls_playlist_from_keyframes_pts(
    keyframes_pts: &[i64],
    total_duration_pts: i64,
    desired_segment_length_pts: i64,
    time_base_num: i64,
    time_base_den: i64,
    endpoint_prefix: &str,
    query_string: &str,
) -> Result<String, String> {
    if time_base_num <= 0 || time_base_den <= 0 {
        return Err("time_base must be positive".to_string());
    }

    let segments_pts = compute_segments_from_keyframes_pts(
        keyframes_pts,
        total_duration_pts,
        desired_segment_length_pts,
    )?;

    let mut out = String::new();
    out.push_str("#EXTM3U\n");
    out.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
    out.push_str("#EXT-X-VERSION:7\n");

    let mut max_seg_seconds = 0.0f64;
    for &seg_pts in &segments_pts {
        let seconds = (seg_pts as f64) * (time_base_num as f64) / (time_base_den as f64);
        if seconds > max_seg_seconds {
            max_seg_seconds = seconds;
        }
    }

    let target_duration = max_seg_seconds.ceil() as i64;
    out.push_str(&format!("#EXT-X-TARGETDURATION:{}\n", target_duration));
    out.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");

    // fMP4 init segment (no startPts on init).
    out.push_str("#EXT-X-MAP:URI=\"");
    out.push_str(endpoint_prefix);
    out.push_str("init.mp4");
    out.push_str(query_string);
    out.push_str("\"\n");

    let mut current_start_pts: i64 = 0;
    for (i, &seg_pts) in segments_pts.iter().enumerate() {
        let seg_seconds = (seg_pts as f64) * (time_base_num as f64) / (time_base_den as f64);

        out.push_str(&format!("#EXTINF:{:.6}, nodesc\n", seg_seconds));

        let qs = append_query_param(query_string, "startPts", current_start_pts);
        out.push_str(endpoint_prefix);
        out.push_str(&i.to_string());
        out.push_str(".m4s");
        out.push_str(&qs);
        out.push('\n');

        current_start_pts += seg_pts;
    }

    out.push_str("#EXT-X-ENDLIST\n");
    Ok(out)
}

/// Convenience wrapper for ffprobe keyframe timestamps provided as seconds (f64).
/// This matches the user's keyframe probe output (best_effort_timestamp_time/pkt_pts_time).
pub fn create_fmp4_hls_playlist_from_keyframes_seconds(
    keyframes_seconds: &[f64],
    total_duration_seconds: f64,
    desired_segment_length_seconds: f64,
    time_base_num: i64,
    time_base_den: i64,
    endpoint_prefix: &str,
    query_string: &str,
) -> Result<String, String> {
    if time_base_num <= 0 || time_base_den <= 0 {
        return Err("time_base must be positive".to_string());
    }
    if total_duration_seconds <= 0.0 || desired_segment_length_seconds <= 0.0 {
        return Err("Invalid segment length or duration".to_string());
    }

    let mut keyframes_pts: Vec<i64> = keyframes_seconds
        .iter()
        .map(|&s| seconds_to_pts(s, time_base_num, time_base_den))
        .collect();
    keyframes_pts.sort_unstable();
    keyframes_pts.dedup();

    let total_duration_pts = seconds_to_pts(total_duration_seconds, time_base_num, time_base_den);
    let desired_segment_length_pts =
        seconds_to_pts(desired_segment_length_seconds, time_base_num, time_base_den);

    create_fmp4_hls_playlist_from_keyframes_pts(
        &keyframes_pts,
        total_duration_pts,
        desired_segment_length_pts,
        time_base_num,
        time_base_den,
        endpoint_prefix,
        query_string,
    )
}
