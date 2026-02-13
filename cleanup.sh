#!/usr/bin/env bash

set -euo pipefail

if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "sqlite3 is required but not installed" >&2
    exit 1
fi

DB_PATH="${1:-}"

if [[ -z "$DB_PATH" && -n "${DATABASE_URL:-}" ]]; then
    case "$DATABASE_URL" in
        sqlite://*)
            DB_PATH="${DATABASE_URL#sqlite://}"
            ;;
        sqlite:*)
            DB_PATH="${DATABASE_URL#sqlite:}"
            ;;
    esac
fi

if [[ -z "$DB_PATH" ]]; then
    DB_PATH="${LYRA_DATA_DIR:-.lyra}/data.db"
fi

if [[ ! -f "$DB_PATH" ]]; then
    echo "Database file not found: $DB_PATH" >&2
    exit 1
fi

sqlite3 "$DB_PATH" <<'SQL'
PRAGMA foreign_keys = ON;
BEGIN IMMEDIATE;

DELETE FROM watch_progress;
DELETE FROM node_matches;
DELETE FROM node_metadata;
DELETE FROM nodes;
DELETE FROM files;
DELETE FROM metadata;
DELETE FROM metadata_fts5;
DELETE FROM assets;
DELETE FROM user_sessions;
DELETE FROM library_users;

DELETE FROM sqlite_sequence
WHERE name IN ('assets', 'files', 'node_matches', 'metadata', 'watch_progress');

-- NULL means "never scanned", so scanner will pick these up immediately.
UPDATE libraries
SET last_scanned_at = NULL;

COMMIT;
PRAGMA wal_checkpoint(TRUNCATE);
VACUUM;
SQL

echo "Cleanup complete: kept users + libraries, cleared all other migrated tables in $DB_PATH."
echo "Libraries are marked for immediate scan on next server startup."
