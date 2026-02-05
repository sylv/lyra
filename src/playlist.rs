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
/// - `endpoint_prefix` is like "hls1/main/".
/// - `endpoint_suffix` is like ".mp4".
///   and this function will append `startPts=<pts>` for each segment.
pub fn create_fmp4_hls_playlist_from_keyframes_pts(
    keyframes_pts: &[i64],
    total_duration_pts: i64,
    desired_segment_length_pts: i64,
    time_base_num: i64,
    time_base_den: i64,
    endpoint_prefix: &str,
    endpoint_suffix: &str,
) -> Result<String, String> {
    let segments_pts = compute_segments_from_keyframes_pts(
        keyframes_pts,
        total_duration_pts,
        desired_segment_length_pts,
    )?;

    let mut playlist = String::new();
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:7\n");
    playlist.push_str("#EXT-X-TARGETDURATION:7\n");
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
    playlist.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");
    // fMP4 init segment (no startPts on init).
    playlist.push_str(&format!("#EXT-X-MAP:URI=\"{}init.mp4{}\"\n", endpoint_prefix, endpoint_suffix));

    let mut start_pts = 0i64;
    for (i, &seg_pts) in segments_pts.iter().enumerate() {
        let dur_seconds = (seg_pts as f64) * (time_base_num as f64) / (time_base_den as f64);
        playlist.push_str(&format!("#EXTINF:{:.6},\n", dur_seconds));
        playlist.push_str(&format!(
            "{}{}.m4s{}?startPts={}\n",
            endpoint_prefix, i, endpoint_suffix, start_pts
        ));
        start_pts += seg_pts;
    }
    playlist.push_str("#EXT-X-ENDLIST\n");
    Ok(playlist)
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
    endpoint_suffix: &str,
) -> Result<String, String> {
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
        endpoint_suffix,
    )
}

/// Create an fMP4 HLS VOD playlist using fixed segment length in seconds.
pub fn create_fmp4_hls_playlist_fixed_seconds(
    total_duration_seconds: f64,
    desired_segment_length_seconds: f64,
    endpoint_prefix: &str,
    endpoint_suffix: &str,
) -> Result<String, String> {
    if total_duration_seconds <= 0.0 || desired_segment_length_seconds <= 0.0 {
        return Err("Invalid segment length or duration".to_string());
    }

    let mut playlist = String::new();
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:7\n");
    playlist.push_str("#EXT-X-TARGETDURATION:7\n");
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n");
    playlist.push_str("#EXT-X-INDEPENDENT-SEGMENTS\n");
    playlist.push_str(&format!("#EXT-X-MAP:URI=\"{}init.mp4{}\"\n", endpoint_prefix, endpoint_suffix));

    let mut cursor = 0.0f64;
    let mut index = 0u64;
    while cursor < total_duration_seconds {
        let remaining = total_duration_seconds - cursor;
        let dur = remaining.min(desired_segment_length_seconds);
        playlist.push_str(&format!("#EXTINF:{:.6},\n", dur));
        playlist.push_str(&format!("{}{}.m4s{}\n", endpoint_prefix, index, endpoint_suffix));
        cursor += dur;
        index += 1;
    }
    playlist.push_str("#EXT-X-ENDLIST\n");
    Ok(playlist)
}

pub fn seconds_to_pts(seconds: f64, time_base_num: i64, time_base_den: i64) -> i64 {
    let pts = seconds * (time_base_den as f64) / (time_base_num as f64);
    pts.round() as i64
}
