DROP TABLE jobs;

CREATE TABLE jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_kind INTEGER NOT NULL,
    target_id TEXT NOT NULL,
    state INTEGER NOT NULL,
    locked_at INTEGER,
    retry_after INTEGER,
    last_run_at INTEGER NOT NULL DEFAULT 0,
    last_error_message TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    UNIQUE (job_kind, target_id),
    CHECK (state IN (0, 1))
) STRICT;

CREATE INDEX idx_jobs_kind_lock_retry_target
    ON jobs(job_kind, locked_at, retry_after, target_id);
