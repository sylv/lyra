use crate::entities::metadata_source::MetadataSource;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "node_metadata")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub node_id: String,
    pub source: MetadataSource,
    pub provider_id: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub name: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub score_display: Option<String>,
    pub score_normalized: Option<i64>,
    pub first_aired: Option<i64>,
    pub last_aired: Option<i64>,
    pub status: Option<MetadataStatus>,
    #[sea_orm(column_type = "Text", nullable)]
    pub tagline: Option<String>,
    pub next_aired: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::node_metadata_cast::Entity")]
    NodeMetadataCast,
    #[sea_orm(has_many = "super::node_metadata_content_ratings::Entity")]
    NodeMetadataContentRatings,
    #[sea_orm(has_many = "super::node_metadata_genres::Entity")]
    NodeMetadataGenres,
    #[sea_orm(has_many = "super::node_metadata_images::Entity")]
    NodeMetadataImages,
    #[sea_orm(has_many = "super::node_metadata_recommendations::Entity")]
    NodeMetadataRecommendations,
    #[sea_orm(
        belongs_to = "super::nodes::Entity",
        from = "Column::NodeId",
        to = "super::nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Nodes,
}

impl Related<super::nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum MetadataStatus {
    Upcoming = 0,
    Airing = 1,
    Returning = 2,
    Finished = 3,
    Cancelled = 4,
    InTheaters = 5,
    Released = 6,
}
