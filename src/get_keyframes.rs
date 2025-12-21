use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;

pub async fn get_keyframes(path: impl AsRef<Path>) -> Result<Vec<f64>> {
    let path = path.as_ref().to_str().unwrap();
    #[rustfmt::skip]
    let  args = vec![
            "-loglevel", "error",
            "-select_streams", "v:0",
            "-show_entries", "packet=pts_time,flags",
            // "-fflags", "+genpts",
            "-of", "csv=p=0",
            path.as_ref()
    ];

    let mut child = tokio::process::Command::new("ffprobe")
        .stdout(Stdio::piped())
        .args(args)
        .spawn()?;

    let mut keyframes = Vec::new();
    let stdout = child.stdout.take().unwrap();
    let mut reader = tokio::io::BufReader::new(stdout).lines();
    while let Some(line) = reader.next_line().await? {
        let Some((pts_time, flags)) = line.split_once(',') else {
            continue;
        };

        if flags.as_bytes()[0] != b'K' {
            continue;
        };

        let pts_time = pts_time.parse::<f64>().unwrap();
        keyframes.push(pts_time);
    }

    if keyframes.is_empty() {
        return Err(anyhow::anyhow!("no keyframes found"));
    }

    Ok(keyframes)
}
