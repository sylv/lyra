use crate::profiles::{TranscodingProfile, audio::AacAudioProfile, video::CopyVideoProfile};
use crate::segmenter::Segmenter;

use crate::profiles::StreamType;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use easy_ffprobe::{Config, Format, Stream, StreamKinds};
use std::cmp::Ordering;
use std::time::Duration;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

const ROOT_DIR: &str = "test";
pub const TARGET_DURATION: f64 = 5.0;

mod ffmpeg;
mod profiles;
mod segmenter;

#[derive(Clone)]
struct AppState {
    segmenters: Arc<Mutex<HashMap<String, Arc<Segmenter>>>>,
    profiles: Vec<Arc<Box<dyn TranscodingProfile + Send + Sync>>>,
    ffmpeg_path: String,
    ffprobe_path: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let (ffmpeg_path, ffprobe_path) = ffmpeg::get_ffmpeg().await.unwrap();

    // clear segments dir
    let segments_dir = PathBuf::from(".lyra/segments");
    if segments_dir.exists() {
        std::fs::remove_dir_all(&segments_dir).unwrap();
        std::fs::create_dir_all(&segments_dir).unwrap();
    } else {
        std::fs::create_dir_all(&segments_dir).unwrap();
    }

    let profiles: Vec<Arc<Box<dyn TranscodingProfile + Send + Sync>>> = vec![
        Arc::new(Box::new(CopyVideoProfile)),
        Arc::new(Box::new(AacAudioProfile)),
    ];

    let app = Router::new()
        .route("/stream/{file_name}/index.m3u8", get(get_master_playlist))
        .route(
            "/stream/{file_name}/{stream_type}/{stream_idx}/{profile}/index.m3u8",
            get(get_stream_playlist),
        )
        .route(
            "/stream/{file_name}/{stream_type}/{stream_idx}/{profile}/{segment}",
            get(get_segment),
        )
        .layer(CorsLayer::permissive())
        .with_state(AppState {
            segmenters: Arc::new(Mutex::new(HashMap::new())),
            profiles,
            ffmpeg_path,
            ffprobe_path,
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_master_playlist(
    State(state): State<AppState>,
    Path(file_name): Path<String>,
) -> Result<Response, StatusCode> {
    let file_path = format!("{}/{}", ROOT_DIR, file_name);
    let probe_data =
        easy_ffprobe::ffprobe_config(Config::new().ffprobe_bin(&state.ffprobe_path), &file_path)
            .map_err(|err| {
                tracing::error!("Error probing file: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let mut playlist = String::new();
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:7\n\n");

    for stream in probe_data.streams {
        let mut profiles = state
            .profiles
            .iter()
            .filter(|p| p.enable_for(&stream))
            .collect::<Vec<_>>();

        if profiles.is_empty() {
            tracing::warn!(stream = ?stream, "no profiles enabled for stream");
            continue;
        }

        // sort audio first, then video, then other
        profiles.sort_by(|a, b| match (a.stream_type(), b.stream_type()) {
            (StreamType::Video, StreamType::Audio) => Ordering::Less,
            (StreamType::Audio, StreamType::Video) => Ordering::Greater,
            _ => Ordering::Equal,
        });

        for profile in profiles.into_iter() {
            let profile_name = profile.name();
            let stream_type = profile.stream_type();
            let stream_idx = stream.index;

            let playlist_path = format!(
                "{}/{}/{}/index.m3u8",
                stream_type.as_str(),
                stream_idx,
                profile_name,
            );

            // todo: codec, resolution, bandwidth, language, etc.
            // todo: use proper names/languages
            match stream.stream {
                StreamKinds::Video(ref video) => {
                    let mut flags = vec![];
                    flags.push("AUDIO=\"group_audio\"".to_string());
                    let header = format!("#EXT-X-STREAM-INF:{}\n", flags.join(","));
                    playlist.push_str(&header);
                    playlist.push_str(&playlist_path);
                    playlist.push_str("\n\n");
                }
                StreamKinds::Audio(ref audio) => {
                    let mut flags = vec![];
                    flags.push("TYPE=AUDIO".to_string());
                    flags.push("GROUP-ID=\"group_audio\"".to_string());
                    flags.push(format!("NAME=\"audio_{}\"", stream_idx));
                    flags.push("DEFAULT=YES".to_string());
                    flags.push(format!("URI=\"{}\"", playlist_path));
                    let header = format!("#EXT-X-MEDIA:{}", flags.join(","));
                    playlist.push_str(&header);
                    playlist.push_str("\n\n");
                }
                _ => {
                    tracing::warn!(stream = ?stream, "unknown stream type");
                    continue;
                }
            };
        }
    }

    Ok(playlist.into_response())
}

async fn get_stream_playlist(
    State(state): State<AppState>,
    Path((file_name, stream_type, stream_idx, profile_name)): Path<(String, String, u64, String)>,
) -> Result<Response, StatusCode> {
    let file_path = format!("{}/{}", ROOT_DIR, file_name);
    let probe_data =
        easy_ffprobe::ffprobe_config(Config::new().ffprobe_bin(&state.ffprobe_path), &file_path)
            .map_err(|err| {
                tracing::error!("Error probing file: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let Some(stream) = probe_data.streams.iter().find(|s| s.index == stream_idx) else {
        tracing::error!(stream_idx, "stream not found");
        return Err(StatusCode::NOT_FOUND);
    };

    tracing::info!("stream: {:?}", stream);

    let mut playlist = String::new();
    playlist.push_str("#EXTM3U\n");
    playlist.push_str("#EXT-X-VERSION:7\n");
    playlist.push_str(&format!(
        "#EXT-X-TARGETDURATION:{}\n",
        TARGET_DURATION.ceil() as u32
    ));
    playlist.push_str("#EXT-X-MEDIA-SEQUENCE:0\n");
    playlist.push_str("#EXT-X-PLAYLIST-TYPE:VOD\n\n");

    let Some(mut remaining_duration) = get_stream_duration(&stream, &probe_data.format) else {
        panic!("no stream duration found on {:#?}", stream);
    };

    let mut segment_index = 0;

    let target_duration = Duration::from_secs_f64(TARGET_DURATION);
    while remaining_duration > Duration::from_secs(0) {
        let segment_duration = remaining_duration.min(target_duration);
        segment_index += 1;

        let segment_path = format!("{}.ts\n\n", segment_index);
        playlist.push_str(&format!("#EXTINF:{:.2}\n", segment_duration.as_secs_f64()));
        playlist.push_str(segment_path.as_str());

        remaining_duration -= segment_duration;
    }

    playlist.push_str("#EXT-X-ENDLIST\n");

    Ok(playlist.into_response())
}

async fn get_segment(
    State(state): State<AppState>,
    Path((file_name, stream_type, stream_idx, profile_name, segment_name)): Path<(
        String,
        String,
        usize,
        String,
        String,
    )>,
) -> Result<Response, StatusCode> {
    let file_path = format!("{}/{}", ROOT_DIR, file_name);
    let segment_idx = segment_name
        .split('.')
        .next()
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let segment_dir = PathBuf::from(".lyra/segments")
        .join(&file_name)
        .join(&stream_type)
        .join(&profile_name)
        .join(&stream_idx.to_string());

    let segmenter_key = format!("{}:video:{}:{}", file_name, profile_name, stream_idx);

    let segmenter = {
        let mut segmenters = state.segmenters.lock().await;
        if !segmenters.contains_key(&segmenter_key) {
            // todo: this means holding the segmenters lock while we probe the file
            let probe_data = easy_ffprobe::ffprobe_config(
                Config::new().ffprobe_bin(&state.ffprobe_path),
                &file_path,
            )
            .map_err(|err| {
                tracing::error!("Error probing file: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let probe_data = Arc::new(probe_data);
            let profile = state
                .profiles
                .iter()
                .find(|p| p.name() == profile_name)
                .unwrap();

            let segmenter = Segmenter::new(
                state.ffmpeg_path.clone(),
                segment_dir,
                profile.clone(),
                PathBuf::from(file_path.clone()),
                probe_data.streams[stream_idx].clone(),
                stream_idx,
            );
            segmenters.insert(segmenter_key.clone(), Arc::new(segmenter));
        }

        segmenters.get(&segmenter_key).unwrap().clone()
    };

    let mut segment = segmenter.get_segment(segment_idx).await.map_err(|err| {
        tracing::error!("Error getting segment: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut buffer = Vec::new();
    tokio::io::AsyncReadExt::read_to_end(&mut segment, &mut buffer)
        .await
        .unwrap();
    Ok(([(axum::http::header::CONTENT_TYPE, "video/mp2t")], buffer).into_response())
}

fn get_stream_duration(stream: &Stream, format: &Format) -> Option<Duration> {
    let duration = stream.duration();
    if let Some(duration) = duration {
        return Some(duration);
    }

    // todo: this is a hack, for some reason "duration_ts" is not always available
    // but "duration" usually is
    let from_stream = match &stream.stream {
        StreamKinds::Video(v) => {
            if let Some(tags) = &v.tags {
                tags.tags.duration
            } else {
                None
            }
        }
        StreamKinds::Audio(v) => {
            if let Some(tags) = &v.tags {
                tags.tags.duration
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(duration) = from_stream {
        return Some(duration);
    }

    if let Some(duration) = format.duration {
        return Some(duration);
    }

    None
}
