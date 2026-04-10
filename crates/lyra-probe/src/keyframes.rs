use crate::{ProbeData, paths::get_ffprobe_path};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};
use tokio::io::AsyncBufReadExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoKeyframes {
    pub video_stream_index: u32,
    pub time_base_num: i64,
    pub time_base_den: i64,
    pub timestamps: Vec<i64>,
}

impl VideoKeyframes {
    pub fn new(
        video_stream_index: u32,
        time_base_num: i64,
        time_base_den: i64,
        timestamps: Vec<i64>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(time_base_num > 0, "time_base_num must be positive");
        anyhow::ensure!(time_base_den > 0, "time_base_den must be positive");
        anyhow::ensure!(
            timestamps.windows(2).all(|window| window[0] <= window[1]),
            "timestamps must be sorted ascending"
        );

        Ok(Self {
            video_stream_index,
            time_base_num,
            time_base_den,
            timestamps,
        })
    }

    pub fn iter_timestamps(&self) -> impl Iterator<Item = i64> + '_ {
        self.timestamps.iter().copied()
    }

    pub fn iter_seconds(&self) -> impl Iterator<Item = f64> + '_ {
        self.iter_timestamps().map(|pts| self.pts_to_seconds(pts))
    }

    pub fn iter_millis(&self) -> impl Iterator<Item = i64> + '_ {
        self.iter_timestamps().map(|pts| self.pts_to_millis(pts))
    }

    pub fn iter_micros(&self) -> impl Iterator<Item = i64> + '_ {
        self.iter_timestamps().map(|pts| self.pts_to_micros(pts))
    }

    pub fn pts_to_seconds(&self, pts: i64) -> f64 {
        (pts as f64) * (self.time_base_num as f64) / (self.time_base_den as f64)
    }

    pub fn pts_to_millis(&self, pts: i64) -> i64 {
        self.pts_to_scaled_units(pts, 1_000)
    }

    pub fn pts_to_micros(&self, pts: i64) -> i64 {
        self.pts_to_scaled_units(pts, 1_000_000)
    }

    pub fn seconds_to_pts(&self, seconds: f64) -> i64 {
        let pts = seconds * (self.time_base_den as f64) / (self.time_base_num as f64);
        pts.round() as i64
    }

    pub fn segment_start_pts(&self, desired_segment_duration: Duration) -> Vec<i64> {
        let desired_segment_length_pts =
            self.seconds_to_pts(desired_segment_duration.as_secs_f64());

        if desired_segment_length_pts <= 0 {
            return vec![0];
        }

        let mut starts = vec![0];
        let mut desired_cut_time = desired_segment_length_pts;

        for &timestamp in &self.timestamps {
            if timestamp >= desired_cut_time {
                starts.push(timestamp);
                desired_cut_time += desired_segment_length_pts;
            }
        }

        starts
    }

    pub fn segment_start_pts_at(
        &self,
        segment_index: usize,
        desired_segment_duration: Duration,
    ) -> i64 {
        let starts = self.segment_start_pts(desired_segment_duration);
        starts.get(segment_index).copied().unwrap_or_else(|| {
            let desired_pts = self.seconds_to_pts(desired_segment_duration.as_secs_f64());
            desired_pts.saturating_mul(segment_index as i64)
        })
    }

    fn pts_to_scaled_units(&self, pts: i64, units_per_second: i64) -> i64 {
        let numerator = (pts as i128) * (self.time_base_num as i128) * (units_per_second as i128);
        (numerator / (self.time_base_den as i128)) as i64
    }
}

pub async fn extract_keyframes(
    file_path: &Path,
    probe: &ProbeData,
    video_stream_index: u32,
    cancellation_token: Option<&CancellationToken>,
) -> anyhow::Result<Option<VideoKeyframes>> {
    let video_stream = probe
        .video_stream(video_stream_index)
        .with_context(|| format!("video stream {video_stream_index} not found"))?;
    let (time_base_num, time_base_den) = video_stream
        .time_base()
        .context("video stream is missing time_base metadata")?;
    let ffprobe_bin = get_ffprobe_path();
    let cancellation_token = cancellation_token
        .cloned()
        .unwrap_or_else(CancellationToken::new);

    #[rustfmt::skip]
    let args = vec![
        "-loglevel".to_string(), "error".to_string(),
        "-fflags".to_string(), "+genpts".to_string(),
        "-show_entries".to_string(), "packet=stream_index,pts,flags".to_string(),
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

                let mut parts = line.splitn(3, ',');
                let Some(stream_index_str) = parts.next() else {
                    continue;
                };
                let Some(pts_str) = parts.next() else {
                    continue;
                };
                let Some(flags_str) = parts.next() else {
                    continue;
                };

                let Ok(stream_index) = stream_index_str.parse::<u32>() else {
                    continue;
                };
                if stream_index != video_stream_index {
                    continue;
                }

                if !flags_str.as_bytes().first().is_some_and(|value| *value == b'K') {
                    continue;
                }

                if let Ok(pts) = pts_str.parse::<i64>() {
                    keyframes.push(pts);
                }
            },
            _ = cancellation_token.cancelled() => {
                cmd.kill().await.ok();
                return Ok(None);
            }
        }
    }

    Ok(Some(VideoKeyframes::new(
        video_stream_index,
        time_base_num,
        time_base_den,
        keyframes,
    )?))
}

#[cfg(test)]
mod tests {
    use super::VideoKeyframes;
    use std::time::Duration;

    #[test]
    fn keyframe_conversions_preserve_time_base_math() {
        let keyframes = VideoKeyframes::new(3, 1, 24_000, vec![0, 24_000, 48_000]).unwrap();

        assert_eq!(keyframes.pts_to_millis(24_000), 1_000);
        assert_eq!(keyframes.pts_to_micros(48_000), 2_000_000);
        assert_eq!(keyframes.seconds_to_pts(1.5), 36_000);
    }

    #[test]
    fn keyframe_helpers_build_segment_starts() {
        let keyframes = VideoKeyframes::new(0, 1, 1, vec![0, 3, 6, 9, 12, 15]).unwrap();

        assert_eq!(keyframes.segment_start_pts(Duration::from_secs(6)), vec![0, 6, 12]);
    }
}
