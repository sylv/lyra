use crate::entities::{file_probe, files};
use anyhow::Result;
use lyra_probe::ProbeData;
use sea_orm::{DatabaseConnection, EntityTrait};

pub async fn load_cached_probe(
    pool: &DatabaseConnection,
    file_id: &str,
) -> Result<Option<ProbeData>> {
    let maybe_row = file_probe::Entity::find_by_id(file_id).one(pool).await?;
    let Some(row) = maybe_row else {
        return Ok(None);
    };

    match row.get_probe() {
        Ok(output) => Ok(Some(output)),
        Err(error) => {
            tracing::warn!(
                file_id,
                error = %error,
                "failed to decode cached probe payload; probing will be retried"
            );
            Ok(None)
        }
    }
}

pub async fn load_cached_keyframes(
    pool: &DatabaseConnection,
    file_id: &str,
) -> Result<Option<Vec<i64>>> {
    let maybe_row = files::Entity::find_by_id(file_id).one(pool).await?;
    let Some(row) = maybe_row else {
        return Ok(None);
    };
    if row.keyframes_json.is_none() {
        return Ok(None);
    }

    match row.decode_keyframes() {
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
