use crate::entities::metadata;
use async_graphql::SimpleObject;

#[derive(Clone, Debug, SimpleObject)]
pub struct NodeProperties {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub poster_url: Option<String>,
    pub background_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub season_number: Option<i64>,
    pub episode_number: Option<i64>,
    pub runtime_minutes: Option<i64>,
    pub released_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl NodeProperties {
    pub(crate) fn from_metadata(metadata: Option<metadata::Model>) -> Self {
        let Some(metadata) = metadata else {
            return Self {
                description: None,
                rating: None,
                poster_url: None,
                background_url: None,
                thumbnail_url: None,
                season_number: None,
                episode_number: None,
                runtime_minutes: None,
                released_at: None,
                ended_at: None,
                created_at: None,
                updated_at: None,
            };
        };

        Self {
            description: metadata.description,
            rating: metadata.score_normalized.map(|score| score as f64 / 10.0),
            poster_url: metadata
                .poster_asset_id
                .map(|asset_id| format!("/api/assets/{asset_id}")),
            background_url: metadata
                .background_asset_id
                .map(|asset_id| format!("/api/assets/{asset_id}")),
            thumbnail_url: metadata
                .thumbnail_asset_id
                .map(|asset_id| format!("/api/assets/{asset_id}")),
            season_number: metadata.season_number,
            episode_number: metadata.episode_number,
            runtime_minutes: None,
            released_at: metadata.released_at,
            ended_at: metadata.ended_at,
            created_at: Some(metadata.created_at),
            updated_at: Some(metadata.updated_at),
        }
    }
}
