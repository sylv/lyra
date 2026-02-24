use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "file_keyframes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_id: i64,
    pub keyframe_list: Vec<u8>,
    pub generated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::files::Entity",
        from = "Column::FileId",
        to = "super::files::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Files,
}

impl Related<super::files::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Files.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
