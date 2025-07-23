use crate::RequestAuth;
use crate::config::get_config;
use crate::entities::file;
use crate::hls::profiles::StreamType;
use crate::hls::profiles::TranscodingProfile;
use crate::hls::profiles::audio::AacAudioProfile;
use crate::hls::profiles::video::CopyVideoProfile;
use crate::hls::segmenter::Segmenter;
use crate::{AppState, ffmpeg};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use easy_ffprobe::{Config, Format, Stream, StreamKinds};
use sea_orm::EntityTrait;
use std::cmp::Ordering;
use std::time::Duration;
use std::{path::PathBuf, sync::Arc};
use tower_http::cors::CorsLayer;

pub const TARGET_DURATION: f64 = 5.0;

pub mod profiles;
pub mod segmenter;

pub fn get_profiles() -> Vec<Arc<Box<dyn TranscodingProfile + Send + Sync>>> {
    vec![
        Arc::new(Box::new(CopyVideoProfile)),
        Arc::new(Box::new(AacAudioProfile)),
    ]
}

pub fn get_hls_router() -> Router<AppState> {
    let mut router = Router::new()
        .route("/stream/{file_id}/index.m3u8", get(get_master_playlist))
        .route(
            "/stream/{file_id}/{stream_type}/{stream_idx}/{profile}/index.m3u8",
            get(get_stream_playlist),
        )
        .route(
            "/stream/{file_id}/{stream_type}/{stream_idx}/{profile}/{segment}",
            get(get_segment),
        );

    #[cfg(debug_assertions)]
    {
        // helps with testing hls endpoints on other sites (ie, hls demo site)
        router = router.layer(CorsLayer::permissive());
    }

    router
}

async fn get_master_playlist(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path(file_id): Path<i64>,
) -> Result<String, (StatusCode, &'static str)> {
    let config = get_config();
    let file = file::Entity::find_by_id(file_id)
        .one(&state.pool)
        .await
        .map_err(|err| {
            tracing::error!("Error finding file: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error finding file")
        })?
        .ok_or((StatusCode::NOT_FOUND, "File not found"))?;

    if file.unavailable_since.is_some() {
        return Err((StatusCode::NOT_FOUND, "File is unavailable"));
    }

    let backend = config
        .get_backend_by_name(&file.backend_name)
        .ok_or((StatusCode::NOT_FOUND, "Backend not found"))?;
    let file_path = backend.root_dir.join(&file.key);

    let ffprobe_path = ffmpeg::get_ffprobe_path();
    let probe_data =
        easy_ffprobe::ffprobe_config(Config::new().ffprobe_bin(&ffprobe_path), &file_path)
            .map_err(|err| {
                tracing::error!("Error probing file: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Error probing file")
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

    Ok(playlist)
}

async fn get_stream_playlist(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, stream_type, stream_idx, profile_name)): Path<(i64, String, u64, String)>,
) -> Result<String, (StatusCode, &'static str)> {
    let config = get_config();
    let file = file::Entity::find_by_id(file_id)
        .one(&state.pool)
        .await
        .map_err(|err| {
            tracing::error!("Error finding file: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error finding file")
        })?
        .ok_or((StatusCode::NOT_FOUND, "File not found"))?;

    if file.unavailable_since.is_some() {
        return Err((StatusCode::NOT_FOUND, "File is unavailable"));
    }

    let backend = config
        .get_backend_by_name(&file.backend_name)
        .ok_or((StatusCode::NOT_FOUND, "Backend not found"))?;
    let file_path = backend.root_dir.join(&file.key);

    let ffprobe_path = ffmpeg::get_ffprobe_path();
    let probe_data =
        easy_ffprobe::ffprobe_config(Config::new().ffprobe_bin(&ffprobe_path), &file_path)
            .map_err(|err| {
                tracing::error!("Error probing file: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Error probing file")
            })?;

    let Some(stream) = probe_data.streams.iter().find(|s| s.index == stream_idx) else {
        tracing::error!(stream_idx, "stream not found");
        return Err((StatusCode::NOT_FOUND, "Stream not found"));
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

    Ok(playlist)
}

async fn get_segment(
    _user: RequestAuth,
    State(state): State<AppState>,
    Path((file_id, stream_type, stream_idx, profile_name, segment_name)): Path<(
        i64,
        String,
        usize,
        String,
        String,
    )>,
) -> Result<Response, (StatusCode, &'static str)> {
    let config = get_config();
    let file = file::Entity::find_by_id(file_id)
        .one(&state.pool)
        .await
        .map_err(|err| {
            tracing::error!("Error finding file: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error finding file")
        })?
        .ok_or((StatusCode::NOT_FOUND, "File not found"))?;

    if file.unavailable_since.is_some() {
        return Err((StatusCode::NOT_FOUND, "File is unavailable"));
    }

    let backend = config
        .get_backend_by_name(&file.backend_name)
        .ok_or((StatusCode::NOT_FOUND, "Backend not found"))?;
    let file_path = backend.root_dir.join(&file.key);

    let segment_idx = segment_name
        .split('.')
        .next()
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let segment_dir = get_config()
        .get_transcode_cache_dir()
        .join(&file_id.to_string())
        .join(&stream_type)
        .join(&profile_name)
        .join(&stream_idx.to_string());

    let segmenter_key = format!("{}:video:{}:{}", file_id, profile_name, stream_idx);
    let segmenter = {
        let mut segmenters = state.segmenters.lock().await;
        if !segmenters.contains_key(&segmenter_key) {
            // todo: this means holding the segmenters lock while we probe the file
            let ffprobe_path = ffmpeg::get_ffprobe_path();
            let probe_data =
                easy_ffprobe::ffprobe_config(Config::new().ffprobe_bin(&ffprobe_path), &file_path)
                    .map_err(|err| {
                        tracing::error!("Error probing file: {:?}", err);
                        (StatusCode::INTERNAL_SERVER_ERROR, "Error probing file")
                    })?;

            let probe_data = Arc::new(probe_data);
            let profile = state
                .profiles
                .iter()
                .find(|p| p.name() == profile_name)
                .unwrap();

            let ffmpeg_path = ffmpeg::get_ffmpeg_path();
            let segmenter = Segmenter::new(
                ffmpeg_path.clone(),
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
        (StatusCode::INTERNAL_SERVER_ERROR, "Error getting segment")
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
