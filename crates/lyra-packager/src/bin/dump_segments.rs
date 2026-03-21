use anyhow::{Context, Result};
use lyra_packager::{
    BuildOptions, build_package, canonicalize_input_path, profiles::VideoCopyProfile,
};
use lyra_ffprobe::{paths::get_ffprobe_path, probe_keyframes_pts_blocking, probe_output_blocking};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "dump-segments".to_string());
    let input_str = args
        .next()
        .with_context(|| format!("usage: {program} <input-file> <output-dir>"))?;
    let output_str = args
        .next()
        .with_context(|| format!("usage: {program} <input-file> <output-dir>"))?;

    let input = canonicalize_input_path(&input_str)?;
    let output_dir = PathBuf::from(&output_str);
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir: {}", output_dir.display()))?;

    let cache_dir = std::env::temp_dir().join("lyra-dump-segments");
    let options = BuildOptions {
        transcode_cache_dir: cache_dir,
    };

    info!(input = %input.display(), "probing");
    let ffprobe_bin = PathBuf::from(get_ffprobe_path()?);
    let ffprobe_output = probe_output_blocking(&ffprobe_bin, &input)?;
    let keyframes = probe_keyframes_pts_blocking(&ffprobe_bin, &input)?;

    // only use the copy profile so we generate exactly what playback would use
    let profiles: Vec<Arc<dyn lyra_packager::profiles::Profile>> =
        vec![Arc::new(VideoCopyProfile)];
    let package = build_package(&input, &profiles, &options, &ffprobe_output, &keyframes)?;

    // find the video_copy session; it may be absent if the file has no keyframe data
    let session = package
        .sessions()
        .values()
        .find(|s| s.profile_id() == "video_copy")
        .cloned()
        .context("no video_copy session (file may lack keyframe data or have no video stream)")?;

    let segment_count = session.segment_count();
    info!(segments = segment_count, "starting segment dump");

    // init segment
    session.ensure_init().await?;
    let init_path = session
        .wait_for_segment_file("init.mp4", Duration::from_secs(30))
        .await?;
    let dest = output_dir.join("init.mp4");
    tokio::fs::copy(&init_path, &dest).await.with_context(|| {
        format!("failed to copy init.mp4 to {}", dest.display())
    })?;
    info!("wrote init.mp4");

    // generate and copy every segment in order
    for i in 0..segment_count as i64 {
        let name = format!("{i}.m4s");
        session.ensure_segment(i, None).await?;
        let seg_path = session
            .wait_for_segment_file(&name, Duration::from_secs(60))
            .await
            .with_context(|| format!("timed out waiting for segment {i}"))?;

        let dest = output_dir.join(&name);
        tokio::fs::copy(&seg_path, &dest).await.with_context(|| {
            format!("failed to copy {name} to {}", dest.display())
        })?;

        info!(segment = i, total = segment_count, "wrote {name}");
    }

    // write the playlist with URIs rewritten to bare filenames
    let playlist = rewrite_playlist_uris(session.playlist());
    let playlist_path = output_dir.join("index.m3u8");
    tokio::fs::write(&playlist_path, &playlist)
        .await
        .with_context(|| format!("failed to write {}", playlist_path.display()))?;
    info!("wrote index.m3u8");

    info!(output = %output_dir.display(), "done");
    Ok(())
}

// rewrite server-relative segment URIs to bare filenames so the playlist works
// against the local output directory. strips query strings and path prefixes,
// leaving just the filename (e.g. "init.mp4", "0.m4s").
fn rewrite_playlist_uris(playlist: &str) -> String {
    playlist
        .lines()
        .map(|line| {
            // #EXT-X-MAP:URI="..."
            if let Some(rest) = line.strip_prefix("#EXT-X-MAP:URI=\"") {
                let uri = rest.trim_end_matches('"');
                let filename = extract_filename(uri);
                return format!("#EXT-X-MAP:URI=\"{filename}\"");
            }
            // bare segment URI lines (start with '/')
            if line.starts_with('/') {
                return extract_filename(line).to_string();
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_filename(uri: &str) -> &str {
    // strip query string, then take the last path component
    let without_query = uri.split('?').next().unwrap_or(uri);
    without_query.rsplit('/').next().unwrap_or(without_query)
}
