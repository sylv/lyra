use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

const BASE_URL: &str = "https://johnvansickle.com/ffmpeg/releases";
const DOWNLOAD_DIR: &str = ".lyra/ffmpeg";

fn get_platform_url() -> Result<String> {
    let arch = std::env::consts::ARCH;
    let platform = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture: {}", arch)),
    };

    Ok(format!(
        "{}/ffmpeg-release-{}-static.tar.xz",
        BASE_URL, platform
    ))
}

/// (ffmpeg_path, ffprobe_path)
pub async fn get_ffmpeg() -> Result<(String, String)> {
    let download_dir = PathBuf::from(DOWNLOAD_DIR);

    // Check if ffmpeg and ffprobe already exist
    let ffmpeg_path = download_dir.join("ffmpeg");
    let ffprobe_path = download_dir.join("ffprobe");

    if ffmpeg_path.exists() && ffprobe_path.exists() {
        // Both binaries exist, return their paths
        return Ok((
            ffmpeg_path.to_string_lossy().to_string(),
            ffprobe_path.to_string_lossy().to_string(),
        ));
    }

    // Create download directory
    fs::create_dir_all(&download_dir)?;

    // Get platform-specific URL
    let url = get_platform_url()?;

    // Download the archive
    tracing::info!(
        "Downloading ffmpeg from {} for architecture {}",
        url,
        std::env::consts::ARCH
    );
    let response = reqwest::get(&url).await?;
    let bytes = response.bytes().await?;

    // Create a temporary file for the downloaded archive
    let archive_path = download_dir.join("ffmpeg.tar.xz");
    let mut file = tokio::fs::File::create(&archive_path).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;
    drop(file);

    // Extract the archive
    tracing::info!("Extracting ffmpeg archive");
    let archive_file = std::fs::File::open(&archive_path)?;
    let decompressed = xz2::read::XzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decompressed);

    // Extract to a temporary directory first
    let temp_extract_dir = download_dir.join("temp_extract");
    fs::create_dir_all(&temp_extract_dir)?;
    archive.unpack(&temp_extract_dir)?;

    // Find the extracted directory (it should contain a single directory)
    let mut extracted_dir = None;
    for entry in fs::read_dir(&temp_extract_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            extracted_dir = Some(entry.path());
            break;
        }
    }

    let extracted_dir = extracted_dir
        .ok_or_else(|| anyhow::anyhow!("Could not find extracted directory in archive"))?;

    // Move ffmpeg and ffprobe binaries to the download directory
    let source_ffmpeg = extracted_dir.join("ffmpeg");
    let source_ffprobe = extracted_dir.join("ffprobe");

    if !source_ffmpeg.exists() {
        return Err(anyhow::anyhow!(
            "ffmpeg binary not found in extracted archive"
        ));
    }
    if !source_ffprobe.exists() {
        return Err(anyhow::anyhow!(
            "ffprobe binary not found in extracted archive"
        ));
    }

    fs::copy(&source_ffmpeg, &ffmpeg_path)?;
    fs::copy(&source_ffprobe, &ffprobe_path)?;

    // Make binaries executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&ffmpeg_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&ffmpeg_path, perms)?;

        let mut perms = fs::metadata(&ffprobe_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&ffprobe_path, perms)?;
    }

    // Clean up temporary files
    fs::remove_file(&archive_path)?;
    fs::remove_dir_all(&temp_extract_dir)?;

    tracing::info!("ffmpeg and ffprobe successfully downloaded and extracted");

    Ok((
        ffmpeg_path.to_string_lossy().to_string(),
        ffprobe_path.to_string_lossy().to_string(),
    ))
}
