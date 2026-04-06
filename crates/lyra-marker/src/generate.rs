use std::{path::Path, process::Stdio};

use anyhow::{Context, bail};
use lyra_probe::{ProbeData, get_ffmpeg_path};
use rusty_chromaprint::Fingerprinter;
use tokio::task::spawn_blocking;
use tokio::{io::AsyncReadExt, process::Command as TokioCommand};
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::{
    FINGERPRINT_CHANNELS, FINGERPRINT_SAMPLE_RATE, FINGERPRINT_SCAN_RATIO, Fingerprint,
    chromaprint_config,
};

pub async fn fingerprint(
    file_path: &Path,
    probe_data: &ProbeData,
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<Fingerprint>> {
    let duration_seconds = probe_data
        .duration_secs
        .context("missing file duration from probe")?;
    let scan_seconds = duration_seconds * FINGERPRINT_SCAN_RATIO;

    debug!(
        path = %file_path.display(),
        duration_seconds,
        scan_seconds,
        "preparing fingerprint decode"
    );

    let ffmpeg_path = get_ffmpeg_path();
    let (samples_tx, samples_rx) = std::sync::mpsc::channel::<Vec<i16>>();
    let fingerprint_worker = spawn_blocking(move || -> anyhow::Result<Vec<u32>> {
        let config = chromaprint_config();
        let mut printer = Fingerprinter::new(&config);
        printer
            .start(FINGERPRINT_SAMPLE_RATE, FINGERPRINT_CHANNELS)
            .context("initializing fingerprinter")?;

        while let Ok(samples) = samples_rx.recv() {
            if !samples.is_empty() {
                printer.consume(&samples);
            }
        }

        printer.finish();
        Ok(printer.fingerprint().to_vec())
    });

    let mut ffmpeg = TokioCommand::new(&ffmpeg_path)
        .kill_on_drop(true)
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-i")
        .arg(file_path)
        .arg("-map")
        .arg("0:a:0")
        .arg("-vn")
        .arg("-ac")
        .arg(FINGERPRINT_CHANNELS.to_string())
        .arg("-ar")
        .arg(FINGERPRINT_SAMPLE_RATE.to_string())
        .arg("-t")
        .arg(format!("{scan_seconds:.4}"))
        .arg("-f")
        .arg("s16le")
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start ffmpeg for '{}'", file_path.display()))?;

    let mut stdout = ffmpeg
        .stdout
        .take()
        .context("failed to capture ffmpeg stdout")?;
    let mut bytes = [0_u8; 32_768];
    let mut samples = Vec::<i16>::with_capacity(bytes.len() / 2);
    let mut trailing_byte = None::<u8>;

    loop {
        let read = if let Some(cancellation_token) = cancellation_token {
            tokio::select! {
                read = stdout.read(&mut bytes) => read,
                _ = cancellation_token.cancelled() => {
                    let _ = ffmpeg.kill().await;
                    let _ = ffmpeg.wait().await;
                    drop(samples_tx);
                    let _ = fingerprint_worker.await;
                    return Ok(None);
                }
            }
        } else {
            stdout.read(&mut bytes).await
        }
        .with_context(|| format!("failed to read decoded audio for '{}'", file_path.display()))?;

        if read == 0 {
            break;
        }

        let mut offset = 0;
        if let Some(prev) = trailing_byte.take() {
            let sample = i16::from_le_bytes([prev, bytes[0]]);
            samples.push(sample);
            offset = 1;
        }

        let available = read.saturating_sub(offset);
        let pair_count = available / 2;
        for idx in 0..pair_count {
            let i = offset + idx * 2;
            samples.push(i16::from_le_bytes([bytes[i], bytes[i + 1]]));
        }

        if available % 2 == 1 {
            trailing_byte = Some(bytes[offset + pair_count * 2]);
        }

        if !samples.is_empty() {
            samples_tx
                .send(std::mem::take(&mut samples))
                .with_context(|| {
                    format!("fingerprint worker dropped for '{}'", file_path.display())
                })?;
        }
    }

    let output = ffmpeg.wait_with_output();
    tokio::pin!(output);
    let output = if let Some(cancellation_token) = cancellation_token {
        tokio::select! {
            output = &mut output => output?,
            _ = cancellation_token.cancelled() => {
                drop(samples_tx);
                let _ = fingerprint_worker.await;
                return Ok(None);
            },
        }
    } else {
        output.await?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "ffmpeg failed for '{}': {}",
            file_path.display(),
            stderr.trim()
        );
    }

    if trailing_byte.is_some() {
        bail!(
            "ffmpeg returned truncated PCM data for '{}'",
            file_path.display()
        );
    }

    drop(samples_tx);
    let fingerprint = fingerprint_worker
        .await
        .context("fingerprint worker failed to join")??;
    Ok(Some(Fingerprint::from_values(&fingerprint)))
}
