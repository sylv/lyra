DELETE FROM watch_progress;
DELETE FROM node_files;
DELETE FROM file_probe;
DELETE FROM file_assets;

DROP TABLE files;

CREATE TABLE files (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    height INTEGER,
    width INTEGER,
    edition_name TEXT,
    audio_fingerprint BLOB NOT NULL,
    segments_json BLOB NOT NULL,
    keyframes_json BLOB NOT NULL,
    unavailable_at INTEGER,
    scanned_at INTEGER,
    discovered_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX files_library_unavailable_scanned_idx
    ON files(library_id, unavailable_at, scanned_at);
