use crate::paths::get_ffprobe_path;
use anyhow::Context;
use std::path::Path;
use tokio::io::AsyncBufReadExt;
use tokio_util::sync::CancellationToken;

pub async fn extract_keyframes(
    file_path: &Path,
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<Vec<i64>>> {
    let ffprobe_bin = get_ffprobe_path();
    let cancellation_token = cancellation_token
        .cloned()
        .unwrap_or_else(CancellationToken::new);

    #[rustfmt::skip]
    let args = vec![
        "-loglevel".to_string(), "error".to_string(),
        "-select_streams".to_string(), "v:0".to_string(),
        "-fflags".to_string(), "+genpts".to_string(),
        "-show_entries".to_string(), "packet=pts,flags".to_string(),
        "-of".to_string(), "csv=print_section=0".to_string(),
    ];

    let mut cmd = tokio::process::Command::new(ffprobe_bin)
        .args(args)
        .arg(file_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .context("failed to spawn ffprobe process")?;

    let stdout = cmd
        .stdout
        .take()
        .context("failed to capture ffprobe stdout")?;

    let reader = tokio::io::BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut keyframes = Vec::new();
    loop {
        tokio::select! {
            line_result = lines.next_line() => {
                let line = match line_result.context("failed to read line from ffprobe output")? {
                    Some(line) => line,
                    None => break,
                };

                let (pts_str, flags_str) = line.split_once(',')
                    .context("failed to parse ffprobe output line")?;

                if flags_str.as_bytes()[0] == b'K' {
                    if let Ok(pts) = pts_str.parse::<i64>() {
                        keyframes.push(pts);
                    }
                }
            },
            _ = cancellation_token.cancelled() => {
                cmd.kill().await.ok();
                return Ok(None);
            }
        }
    }

    Ok(Some(keyframes))
}
