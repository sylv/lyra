use crate::entities::{metadata_source::MetadataSource, node_metadata, nodes};
use sea_orm::{
    ColumnTrait, Condition, JoinType, QueryFilter, QuerySelect, RelationTrait, Select,
    sea_query::{Alias, Expr, Query},
};

pub fn join_preferred_node_metadata(mut query: Select<nodes::Entity>) -> Select<nodes::Entity> {
    query = query
        .join(JoinType::LeftJoin, nodes::Relation::NodeMetadata.def())
        .filter(preferred_node_metadata_condition());
    query
}

fn preferred_node_metadata_condition() -> Condition {
    let remote_rows = Alias::new("preferred_remote_rows");

    Condition::any()
        .add(node_metadata::Column::Id.is_null())
        .add(node_metadata::Column::Source.eq(MetadataSource::Remote))
        .add(
            Condition::all()
                .add(node_metadata::Column::Source.eq(MetadataSource::Local))
                .add(
                    Condition::all().not().add(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from_as(node_metadata::Entity, remote_rows.clone())
                            .and_where(
                                Expr::col((remote_rows.clone(), node_metadata::Column::NodeId))
                                    .equals((node_metadata::Entity, node_metadata::Column::NodeId)),
                            )
                            .and_where(
                                Expr::col((remote_rows, node_metadata::Column::Source))
                                    .eq(MetadataSource::Remote),
                            )
                            .to_owned(),
                    )),
                ),
        )
}
