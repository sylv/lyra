ALTER TABLE jobs ADD COLUMN locked_at INTEGER;
ALTER TABLE jobs ADD COLUMN priority_at INTEGER;

CREATE INDEX idx_jobs_kind_file_id ON jobs(job_kind, file_id);
CREATE INDEX idx_jobs_kind_asset_id ON jobs(job_kind, asset_id);
CREATE INDEX idx_jobs_kind_node_id ON jobs(job_kind, node_id);
CREATE INDEX idx_jobs_kind_priority_run_after ON jobs(job_kind, priority_at, run_after, id);
