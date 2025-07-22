use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub transcode_cache_dir: Option<PathBuf>,
    pub image_dir: Option<PathBuf>,
    pub ffmpeg_dir: Option<PathBuf>,
    pub backends: Vec<Backend>,
    pub tmdb_api_key: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn get_transcode_cache_dir(&self) -> PathBuf {
        if let Some(dir) = self.transcode_cache_dir.as_ref() {
            dir.clone()
        } else {
            self.data_dir.join("transcode_cache")
        }
    }

    pub fn get_image_dir(&self) -> PathBuf {
        if let Some(dir) = self.image_dir.as_ref() {
            dir.clone()
        } else {
            self.data_dir.join("image_cache")
        }
    }

    pub fn get_ffmpeg_dir(&self) -> PathBuf {
        if let Some(dir) = self.ffmpeg_dir.as_ref() {
            dir.clone()
        } else {
            self.data_dir.join("ffmpeg_cache")
        }
    }

    pub fn get_backend_by_name(&self, name: &str) -> Option<&Backend> {
        self.backends.iter().find(|b| b.name == name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "type")]
pub struct Backend {
    pub name: String,
    pub root_dir: PathBuf,
}

static CONFIG: once_cell::sync::Lazy<Config> =
    once_cell::sync::Lazy::new(|| load_config().expect("failed to load lyra config"));

pub fn get_config() -> &'static Config {
    &CONFIG
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config = config::Config::builder()
        .add_source(config::Environment::with_prefix("lyra"))
        .add_source(config::File::with_name("lyra"))
        .set_default("data_dir", ".lyra")?
        .set_default("tmdb_api_key", "f81a38fe9eba82e5dc3695a7406068bd")?
        .set_default("host", "127.0.0.1")?
        .set_default("port", "8000")?
        .build()
        .unwrap();

    let config: Config = config.try_deserialize()?;
    assert!(!config.backends.is_empty(), "no backends configured");

    std::fs::create_dir_all(&config.data_dir)?;
    std::fs::create_dir_all(&config.get_transcode_cache_dir())?;
    std::fs::create_dir_all(&config.get_image_dir())?;
    std::fs::create_dir_all(&config.get_ffmpeg_dir())?;

    Ok(config)
}
