use anyhow::{Context, Result, bail};
use lyra_timeline_preview::{PreviewOptions, generate_previews};
use std::{env, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let input_file: PathBuf = env::args()
        .nth(1)
        .context("Please provide a video file path as the first argument")?
        .into();

    if !input_file.exists() {
        bail!("Input file does not exist: {:?}", input_file);
    }
    let options = PreviewOptions {
        ..PreviewOptions::default()
    };

    let previews = generate_previews(&input_file, &options, None)
        .await?
        .expect("timeline preview CLI should not cancel");
    for (index, preview) in previews.iter().enumerate() {
        let output_path = env::current_dir()?.join(format!("generated_preview_{}.webp", index + 1));
        tokio::fs::write(&output_path, &preview.preview_bytes).await?;
        println!("wrote {}", output_path.display());
    }
    println!("generated {} preview sheet(s)", previews.len());

    Ok(())
}
