use ed25519_dalek::SigningKey;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const SIGNING_KEY_FILENAME: &str = "signing_key";

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

    pub fn get_signing_key_path(&self) -> PathBuf {
        self.data_dir.join(SIGNING_KEY_FILENAME)
    }
}

static CONFIG: once_cell::sync::Lazy<(Config, SigningKey)> =
    once_cell::sync::Lazy::new(|| init().expect("failed to load lyra config"));

pub fn get_config() -> &'static Config {
    &CONFIG.0
}

pub fn get_signing_key() -> &'static SigningKey {
    &CONFIG.1
}

fn init() -> Result<(Config, SigningKey), Box<dyn std::error::Error>> {
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

    tracing::info!(
        data_dir = ?config.data_dir,
        host = %config.host,
        port = config.port,
        "loaded config"
    );

    let signing_key_path = config.get_signing_key_path();
    let signing_key = if signing_key_path.exists() {
        tracing::info!("loading signing key from {}", signing_key_path.display());
        let key_bytes = std::fs::read(&signing_key_path)?;
        if key_bytes.len() != 32 {
            return Err(format!("signing_key must be 32 bytes, got {}", key_bytes.len()).into());
        }

        let mut key_bytes_array = [0_u8; 32];
        key_bytes_array.copy_from_slice(&key_bytes);
        SigningKey::from_bytes(&key_bytes_array)
    } else {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        std::fs::write(&signing_key_path, signing_key.to_bytes())?;
        signing_key
    };

    Ok((config, signing_key))
}
