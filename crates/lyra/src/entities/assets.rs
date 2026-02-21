use async_graphql::Enum;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "assets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub source: AssetSource,
    #[sea_orm(column_type = "Text", nullable)]
    pub source_url: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub hash_sha256: Option<String>,
    pub size_bytes: Option<i64>,
    #[sea_orm(column_type = "Text", nullable)]
    pub mime_type: Option<String>,
    pub height: Option<i64>,
    pub width: Option<i64>,
    #[sea_orm(column_type = "Blob", nullable)]
    pub thumbhash: Option<Vec<u8>>,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(
    Debug, Enum, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i64", db_type = "Integer")]
pub enum AssetSource {
    Local = 0,
}
