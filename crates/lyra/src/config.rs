use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub transcode_cache_dir: Option<PathBuf>,
    pub image_dir: Option<PathBuf>,
    pub asset_store_dir: Option<PathBuf>,
    pub host: String,
    pub port: u16,
    pub clear_transcode_cache_on_start: bool,
    pub library_scan_interval: i64,
    pub watch_progress_minimum_threshold: f32,
    pub watch_progress_completed_threshold: f32,
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

    pub fn get_asset_store_dir(&self) -> PathBuf {
        if let Some(dir) = self.asset_store_dir.as_ref() {
            dir.clone()
        } else {
            self.data_dir.join("assets")
        }
    }

    pub fn get_tmp_dir(&self) -> PathBuf {
        self.data_dir.join("tmp")
    }
}

static CONFIG: once_cell::sync::Lazy<Config> =
    once_cell::sync::Lazy::new(|| load_config().expect("failed to load lyra config"));

pub fn get_config() -> &'static Config {
    &CONFIG
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config = config::Config::builder()
        .add_source(config::Environment::with_prefix("lyra"))
        .add_source(config::File::with_name("lyra.yml").required(false))
        .set_default("data_dir", ".lyra")?
        .set_default("host", "127.0.0.1")?
        .set_default("port", "8000")?
        .set_default("clear_transcode_cache_on_start", false)?
        .set_default("library_scan_interval", 4 * 60 * 60)? // 4 hours
        .set_default("watch_progress_minimum_threshold", 0.05)?
        .set_default("watch_progress_completed_threshold", 0.8)?
        .build()
        .unwrap();

    let config: Config = config.try_deserialize()?;
    tracing::info!("loaded config: {:?}", config);

    if config.clear_transcode_cache_on_start && config.get_transcode_cache_dir().exists() {
        tracing::info!("clearing transcode cache");
        std::fs::remove_dir_all(&config.get_transcode_cache_dir())?;
    }

    let temp_dir = config.get_tmp_dir();
    if temp_dir.exists() {
        tracing::info!("clearing temp directory");
        std::fs::remove_dir_all(&temp_dir)?;
    }

    std::fs::create_dir_all(&config.data_dir)?;
    std::fs::create_dir_all(&config.get_transcode_cache_dir())?;
    std::fs::create_dir_all(&config.get_image_dir())?;
    std::fs::create_dir_all(&config.get_asset_store_dir())?;
    std::fs::create_dir_all(&config.get_tmp_dir())?;

    Ok(config)
}
