#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 1 ]]; then
    echo "usage: $0 [data-dir]" >&2
    exit 1
fi

if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "sqlite3 is required" >&2
    exit 1
fi

data_dir="${1:-${LYRA_DATA_DIR:-.lyra}}"
db_path="${data_dir}/data.db"
asset_store_dir="${LYRA_ASSET_STORE_DIR:-${data_dir}/assets}"
image_dir="${LYRA_IMAGE_DIR:-${data_dir}/image_cache}"
transcode_cache_dir="${LYRA_TRANSCODE_CACHE_DIR:-${data_dir}/transcode_cache}"
tmp_dir="${data_dir}/tmp"

if [[ ! -f "${db_path}" ]]; then
    echo "database not found at ${db_path}" >&2
    exit 1
fi

echo "Resetting Lyra database at ${db_path}"

sqlite3 "${db_path}" >/dev/null <<'SQL'
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;

BEGIN IMMEDIATE;

-- Preserve library assignments so existing users keep access to the libraries we keep.
DELETE FROM user_sessions;
DELETE FROM watch_progress;
DELETE FROM jobs;
DELETE FROM file_probe;
DELETE FROM file_assets;
DELETE FROM node_metadata;
DELETE FROM node_files;
DELETE FROM node_closure;
DELETE FROM nodes;
DELETE FROM files;
DELETE FROM assets;
DELETE FROM node_search_fts;

UPDATE libraries
SET last_scanned_at = NULL;

DELETE FROM sqlite_sequence
WHERE name = 'jobs';

COMMIT;

PRAGMA wal_checkpoint(TRUNCATE);
VACUUM;
SQL

rm -rf "${asset_store_dir}" "${image_dir}" "${transcode_cache_dir}" "${tmp_dir}"
mkdir -p "${asset_store_dir}" "${image_dir}" "${transcode_cache_dir}" "${tmp_dir}"

echo "Reset complete."
