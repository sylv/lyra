use lyra_probe::ProbeData;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "file_probe")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub file_id: String,
    #[sea_orm(column_type = "Blob")]
    pub probe: Vec<u8>,
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

impl Model {
    pub fn get_probe(&self) -> anyhow::Result<ProbeData> {
        lyra_probe::decode_probe_data_json_zstd(&self.probe)
    }
}
