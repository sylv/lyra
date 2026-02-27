use crate::{
    entities::{file_keyframes, file_probe},
    json_encoding,
};
use anyhow::Result;
use lyra_ffprobe::FfprobeOutput;
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn load_cached_ffprobe_output(
    pool: &DatabaseConnection,
    file_id: i64,
) -> Result<Option<FfprobeOutput>> {
    let maybe_row = file_probe::Entity::find_by_id(file_id).one(pool).await?;
    let Some(row) = maybe_row else {
        return Ok(None);
    };

    match row.decode_ffprobe_output() {
        Ok(output) => Ok(Some(output)),
        Err(error) => {
            tracing::warn!(
                file_id,
                error = %error,
                "failed to decode cached ffprobe payload; probing will be retried"
            );
            Ok(None)
        }
    }
}

pub async fn load_cached_keyframes(
    pool: &DatabaseConnection,
    file_id: i64,
) -> Result<Option<Vec<i64>>> {
    let maybe_row = file_keyframes::Entity::find_by_id(file_id)
        .one(pool)
        .await?;
    let Some(row) = maybe_row else {
        return Ok(None);
    };

    match json_encoding::decode_json_zstd::<Vec<i64>>(&row.keyframe_list) {
        Ok(keyframes) => Ok(Some(keyframes)),
        Err(error) => {
            tracing::warn!(
                file_id,
                error = %error,
                "failed to decode cached keyframe payload; probing will be retried"
            );
            Ok(None)
        }
    }
}
