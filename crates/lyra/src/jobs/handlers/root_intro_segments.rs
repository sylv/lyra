use crate::{
    entities::{files, jobs as jobs_entity, libraries, node_files, nodes, nodes::NodeKind},
    file_analysis,
    jobs::{Job, JobLease, JobOutcome, JobScheduling},
    json_encoding,
    segment_markers::intro_segment_from_range,
};
use anyhow::Context;
use lyra_marker::{Fingerprint, detect_intros, fingerprint};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
    sea_query::Expr,
};
use sqlx::{QueryBuilder, Sqlite};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Debug, Default)]
pub struct RootIntroSegmentsJob;

#[derive(Clone, Debug)]
struct RootFile {
    file_id: String,
    file_path: PathBuf,
    fingerprint: Option<Fingerprint>,
}

#[derive(Debug, FromQueryResult)]
struct RootFileQueryRow {
    file_id: String,
    relative_path: String,
    library_path: String,
    audio_fingerprint: Option<Vec<u8>>,
}

#[async_trait::async_trait]
impl Job for RootIntroSegmentsJob {
    type Entity = nodes::Entity;
    type Model = nodes::Model;

    const JOB_KIND: jobs_entity::JobKind = jobs_entity::JobKind::NodeGenerateIntroSegments;
    const SCHEDULING: JobScheduling = JobScheduling::Heavy(2);

    fn query(&self) -> Select<Self::Entity> {
        nodes::Entity::find()
            .filter(nodes::Column::ParentId.is_null())
            .filter(nodes::Column::Kind.eq(NodeKind::Series))
            .filter(
                Condition::any()
                    .add(nodes::Column::LastFingerprintVersion.is_null())
                    .add(
                        Expr::col(nodes::Column::LastFingerprintVersion)
                            .ne(Expr::col(nodes::Column::LastAddedAt)),
                    ),
            )
            .order_by_asc(nodes::Column::Id)
    }

    fn target_id(&self, target: &Self::Model) -> String {
        target.id.clone()
    }

    async fn run(
        &self,
        db: &DatabaseConnection,
        root: Self::Model,
        ctx: &JobLease,
    ) -> anyhow::Result<JobOutcome> {
        let mut root_files = load_root_files(db, &root.id).await?;
        let mut detection_inputs = Vec::with_capacity(root_files.len());
        for file in &mut root_files {
            if ctx.is_cancelled() {
                return Ok(JobOutcome::Cancelled);
            }

            let fingerprint = match file.fingerprint.clone() {
                Some(fingerprint) => fingerprint,
                None => {
                    let probe_data = file_analysis::load_cached_probe(db, &file.file_id)
                        .await?
                        .with_context(|| {
                            format!("missing cached probe data for file {}", file.file_id)
                        })?;
                    let Some(fingerprint) =
                        fingerprint(&file.file_path, &probe_data, ctx.get_cancellation_token())
                            .await?
                    else {
                        return Ok(JobOutcome::Cancelled);
                    };

                    store_audio_fingerprint(db, &file.file_id, fingerprint.as_bytes()).await?;
                    file.fingerprint = Some(fingerprint.clone());
                    fingerprint
                }
            };

            detection_inputs.push((file.file_path.clone(), fingerprint));
        }

        let Some(detections) =
            detect_intros(&detection_inputs, ctx.get_cancellation_token()).await?
        else {
            return Ok(JobOutcome::Cancelled);
        };

        let detections_by_path = detections
            .into_iter()
            .map(|detection| (detection.path.clone(), detection))
            .collect::<HashMap<_, _>>();

        let mut segment_updates = Vec::with_capacity(root_files.len());
        for file in &root_files {
            let detection = detections_by_path.get(&file.file_path).with_context(|| {
                format!(
                    "intro detection output missing file '{}'",
                    file.file_path.display()
                )
            })?;
            let segments = detection
                .intro
                .and_then(intro_segment_from_range)
                .into_iter()
                .collect::<Vec<_>>();

            let payload = json_encoding::encode_json_zstd(&segments).with_context(|| {
                format!("failed to encode intro segments for file {}", file.file_id)
            })?;
            segment_updates.push((file.file_id.clone(), payload));
        }
        store_segments_bulk(db, &segment_updates).await?;

        if root.last_fingerprint_version != Some(root.last_added_at) {
            store_last_fingerprint_version(db, &root.id, Some(root.last_added_at)).await?;
        }

        Ok(JobOutcome::Complete)
    }
}

async fn load_root_files(db: &DatabaseConnection, root_id: &str) -> anyhow::Result<Vec<RootFile>> {
    let rows = node_files::Entity::find()
        .join(JoinType::InnerJoin, node_files::Relation::Nodes.def())
        .join(JoinType::InnerJoin, node_files::Relation::Files.def())
        .join(JoinType::InnerJoin, files::Relation::Libraries.def())
        .filter(nodes::Column::RootId.eq(root_id.to_string()))
        .filter(nodes::Column::Kind.eq(NodeKind::Episode))
        .filter(files::Column::UnavailableAt.is_null())
        .select_only()
        .column_as(files::Column::Id, "file_id")
        .column_as(files::Column::RelativePath, "relative_path")
        .column_as(libraries::Column::Path, "library_path")
        .column_as(files::Column::AudioFingerprint, "audio_fingerprint")
        .order_by_asc(nodes::Column::Order)
        .order_by_asc(files::Column::Id)
        .into_model::<RootFileQueryRow>()
        .all(db)
        .await?;

    let mut unique_rows = Vec::new();
    let mut seen_file_ids = HashSet::new();
    for row in rows {
        if seen_file_ids.insert(row.file_id.clone()) {
            unique_rows.push(row);
        }
    }

    let mut output = Vec::with_capacity(unique_rows.len());
    for row in unique_rows {
        let file_id = row.file_id;
        output.push(RootFile {
            file_id: file_id.clone(),
            file_path: PathBuf::from(row.library_path).join(row.relative_path),
            fingerprint: row
                .audio_fingerprint
                .map(Fingerprint::from_bytes)
                .transpose()
                .with_context(|| format!("invalid stored fingerprint for file {}", file_id))?,
        });
    }

    Ok(output)
}

async fn store_audio_fingerprint(
    db: &impl ConnectionTrait,
    file_id: &str,
    fingerprint: &[u8],
) -> anyhow::Result<()> {
    files::Entity::update(files::ActiveModel {
        id: Set(file_id.to_string()),
        audio_fingerprint: Set(Some(fingerprint.to_vec())),
        ..Default::default()
    })
    .exec(db)
    .await?;

    Ok(())
}

async fn store_segments_bulk(
    db: &DatabaseConnection,
    updates: &[(String, Vec<u8>)],
) -> anyhow::Result<()> {
    if updates.is_empty() {
        return Ok(());
    }

    let mut builder: QueryBuilder<'_, Sqlite> =
        QueryBuilder::new("UPDATE files SET segments_json = CASE id");
    for (file_id, payload) in updates {
        builder.push(" WHEN ");
        builder.push_bind(file_id);
        builder.push(" THEN ");
        builder.push_bind(payload);
    }
    builder.push(" END WHERE id IN (");

    let mut separated = builder.separated(", ");
    for (file_id, _) in updates {
        separated.push_bind(file_id);
    }
    separated.push_unseparated(")");

    builder
        .build()
        .execute(db.get_sqlite_connection_pool())
        .await?;

    Ok(())
}

async fn store_last_fingerprint_version(
    db: &impl ConnectionTrait,
    root_id: &str,
    version: Option<i64>,
) -> anyhow::Result<()> {
    nodes::Entity::update(nodes::ActiveModel {
        id: Set(root_id.to_string()),
        last_fingerprint_version: Set(version),
        ..Default::default()
    })
    .exec(db)
    .await?;

    Ok(())
}
