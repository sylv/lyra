use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use lyra_marker::{IntroDetectionInputFile, detect_intros};
use lyra_probe::probe;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        eprintln!("usage: lyra-marker <directory>");
        return Ok(());
    }

    let input_dir = Path::new(&args[1]);
    if !input_dir.is_dir() {
        bail!(
            "'{}' is not a directory",
            input_dir.as_os_str().to_string_lossy()
        );
    }

    let mut input_files = collect_input_files(input_dir)?;
    input_files.sort();

    let mut detection_inputs = Vec::with_capacity(input_files.len());
    for path in &input_files {
        detection_inputs.push(IntroDetectionInputFile {
            path: path.clone(),
            probe_data: probe(path).await?,
            fingerprint_cache: None,
        });
    }

    let intros = detect_intros(&detection_inputs, None)
        .await?
        .context("intro detection cancelled unexpectedly")?;
    for detection in intros {
        println!("{}", detection.path.display());
        if let Some(intro) = detection.intro {
            println!(
                "  {} -- {}",
                format_to_duration(f64::from(intro.start_seconds)),
                format_to_duration(f64::from(intro.end_seconds)),
            );
        } else {
            println!("  none");
        }
    }

    Ok(())
}

fn collect_input_files(input_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut dirs = vec![input_dir.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("failed to read '{}'", dir.display()))?
        {
            let entry =
                entry.with_context(|| format!("failed to read entry in '{}'", dir.display()))?;
            let path = entry.path();

            if path.is_dir() {
                dirs.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }

    Ok(files)
}

fn format_to_duration(seconds: f64) -> String {
    let centiseconds = (seconds * 100.0).round() as u64;
    let total_secs = centiseconds / 100;
    let hours = total_secs / 3600;
    let rem = total_secs % 3600;
    let minutes = rem / 60;
    let secs = rem % 60;
    let fraction = centiseconds % 100;

    format!("{hours}:{minutes:02}:{secs:02}.{fraction:02}")
}
