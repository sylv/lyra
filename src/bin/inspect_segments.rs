use anyhow::{Context, Result};
use glob::glob;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    process::Command,
};

const SEGMENT_ROOT: &str = "/tmp/lyra-hls";

#[derive(Debug)]
struct SegmentTiming {
    id: usize,
    start: f64,
    duration: f64,
    end: f64,
    path: PathBuf,
}

fn main() -> Result<()> {
    let root = PathBuf::from(SEGMENT_ROOT);
    if !root.exists() {
        anyhow::bail!("segment root {} does not exist", root.display());
    }

    let mut segments = collect_segments(&root)?;
    if segments.is_empty() {
        println!("No segments found under {}", root.display());
        return Ok(());
    }

    segments.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then_with(|| a.start.partial_cmp(&b.start).unwrap_or(Ordering::Equal))
    });

    println!(
        "Found {} segment(s) under {}",
        segments.len(),
        root.display()
    );

    for seg in segments {
        let rel_path = seg.path.strip_prefix(&root).unwrap_or(&seg.path);
        println!(
            "segment {:>4} | start {:>9.3} | end {:>9.3} | dur {:>7.3} | {}",
            seg.id,
            seg.start,
            seg.end,
            seg.duration,
            rel_path.display(),
        );
    }

    Ok(())
}

fn collect_segments(root: &Path) -> Result<Vec<SegmentTiming>> {
    let pattern = format!("{}/**/segment_*.m4s", root.display());
    let mut segments = Vec::new();

    for entry in glob(&pattern).context("failed to read segment glob")? {
        let path = entry.context("failed to read segment path")?;
        let Some(id) = extract_segment_id(&path) else {
            continue;
        };

        // Enforce directory-to-file alignment: seg_X should only contain segment_X.m4s
        if let Some(parent) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
        {
            if let Some(dir_id) = parent
                .strip_prefix("seg_")
                .and_then(|s| s.parse::<usize>().ok())
            {
                if dir_id != id {
                    // Skip mismatched segment file (likely incomplete/partial)
                    continue;
                }
            }
        }

        match probe_segment(&path) {
            Ok((start, duration)) => segments.push(SegmentTiming {
                id,
                start,
                duration,
                end: start + duration,
                path,
            }),
            Err(err) => {
                eprintln!("Skipping {}: {err:#}", path.display());
            }
        }
    }

    Ok(segments)
}

fn extract_segment_id(path: &Path) -> Option<usize> {
    let filename = path.file_stem()?.to_str()?;
    filename
        .strip_prefix("segment_")
        .and_then(|rest| rest.parse::<usize>().ok())
}

#[derive(Deserialize)]
struct Frame {
    #[serde(default)]
    pts_time: String,
    #[serde(default)]
    duration_time: String,
    #[serde(default)]
    pkt_duration_time: String,
}

#[derive(Deserialize)]
struct ProbeFrames {
    frames: Vec<Frame>,
}

fn probe_segment(path: &Path) -> Result<(f64, f64)> {
    let init = path.parent().map(|p| p.join("init.mp4"));
    // Use concat protocol to let ffprobe see init + fragment as one stream.
    let concat_input = init
        .as_ref()
        .filter(|p| p.exists())
        .map(|init_path| {
            format!(
                "concat:{}|{}",
                init_path.to_string_lossy(),
                path.to_string_lossy()
            )
        })
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let args = vec![
        "-v".into(),
        "quiet".into(),
        "-print_format".into(),
        "json".into(),
        "-show_frames".into(),
        "-show_entries".into(),
        "frame=pts_time,duration_time".into(),
        "-select_streams".into(),
        "v".into(),
        "-i".into(),
        concat_input.clone(),
    ];

    let output = Command::new("ffprobe")
        .args(&args)
        .output()
        .with_context(|| format!("failed to run ffprobe on {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "ffprobe failed on {} (code {:?})\nstderr: {}\nstdout: {}",
            path.display(),
            output.status.code(),
            stderr.trim(),
            stdout.trim()
        );
    }

    let parsed: ProbeFrames =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe frame output")?;

    if parsed.frames.is_empty() {
        anyhow::bail!(
            "no frames returned by ffprobe for {} (init present: {}, input={})",
            path.display(),
            init.as_ref().is_some_and(|p| p.exists()),
            concat_input
        );
    }

    let mut start: Option<f64> = None;
    let mut end: Option<f64> = None;
    for frame in parsed.frames {
        let pts = frame.pts_time.parse::<f64>().unwrap_or(0.0);
        let dur = frame
            .duration_time
            .parse::<f64>()
            .ok()
            .or_else(|| frame.pkt_duration_time.parse::<f64>().ok())
            .unwrap_or(0.0);
        start = Some(start.map_or(pts, |s| s.min(pts)));
        let frame_end = pts + dur;
        end = Some(end.map_or(frame_end, |e| e.max(frame_end)));
    }

    let start = start.unwrap_or(0.0);
    let end = end.unwrap_or(start);
    let duration = (end - start).max(0.0);

    Ok((start, duration))
}
