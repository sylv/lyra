use lyra_probe::probe;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "ts", "m2ts", "mts",
];

async fn write_sample<T: Serialize>(stem: &str, json_dir: &Path, value: &T) -> anyhow::Result<()> {
    let json_path = json_dir.join(format!("{stem}.json"));
    fs::write(&json_path, serde_json::to_vec(value)?).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let dir = std::env::args()
        .nth(1)
        .expect("Usage: dump-samples <directory>");

    let json_dir = PathBuf::from("samples_json");
    // Clear and recreate the output directory on each run.
    if json_dir.exists() {
        fs::remove_dir_all(&json_dir).await?;
    }
    fs::create_dir_all(&json_dir).await?;

    let mut ok = 0usize;
    let mut errors = 0usize;

    walk_and_probe(Path::new(&dir), &json_dir, &mut ok, &mut errors).await?;

    eprintln!("\nDone — {} processed, {} errors", ok, errors);
    eprintln!("Samples written to: {}", json_dir.display());

    Ok(())
}

// Recursively walk `dir`, probing each video file in place.
async fn walk_and_probe(
    dir: &Path,
    json_dir: &Path,
    ok: &mut usize,
    errors: &mut usize,
) -> anyhow::Result<()> {
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            Box::pin(walk_and_probe(&path, json_dir, ok, errors)).await?;
        } else if file_type.is_file() && is_video(&path) {
            let hash = path_hash(&path);
            let stem = format!("{hash:016x}_probe");
            match probe(&path).await {
                Ok(data) => {
                    write_sample(&stem, json_dir, &data).await?;
                    eprintln!("[probe ok] {}", path.display());
                    *ok += 1;
                }
                Err(e) => {
                    eprintln!("[probe err] {}: {e}", path.display());
                    *errors += 1;
                }
            }
        }
    }
    Ok(())
}

fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn path_hash(path: &Path) -> u64 {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let bytes = canonical.to_string_lossy();
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
