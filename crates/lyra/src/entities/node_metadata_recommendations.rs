use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "node_metadata_recommendations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: String,
    pub node_metadata_id: String,
    pub provider_id: String,
    pub media_kind: RecommendationMediaKind,
    pub tmdb_id: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub imdb_id: Option<String>,
    pub name: String,
    pub first_aired: Option<i64>,
    pub position: i64,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::node_metadata::Entity",
        from = "Column::NodeMetadataId",
        to = "super::node_metadata::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    NodeMetadata,
}

impl Related<super::node_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum RecommendationMediaKind {
    Movie = 0,
    Series = 1,
}
