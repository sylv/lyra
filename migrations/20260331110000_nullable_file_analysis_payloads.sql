PRAGMA foreign_keys = OFF;

CREATE TABLE files_new (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    height INTEGER,
    width INTEGER,
    edition_name TEXT,
    audio_fingerprint BLOB,
    segments_json BLOB,
    keyframes_json BLOB,
    unavailable_at INTEGER,
    scanned_at INTEGER,
    discovered_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
) STRICT;

INSERT INTO files_new (
    id,
    library_id,
    relative_path,
    size_bytes,
    height,
    width,
    edition_name,
    audio_fingerprint,
    segments_json,
    keyframes_json,
    unavailable_at,
    scanned_at,
    discovered_at
)
SELECT
    id,
    library_id,
    relative_path,
    size_bytes,
    height,
    width,
    edition_name,
    CASE
        WHEN length(audio_fingerprint) = 0 THEN NULL
        ELSE audio_fingerprint
    END,
    CASE
        WHEN length(segments_json) = 0 THEN NULL
        ELSE segments_json
    END,
    CASE
        WHEN length(keyframes_json) = 0 THEN NULL
        ELSE keyframes_json
    END,
    unavailable_at,
    scanned_at,
    discovered_at
FROM files;

DROP TABLE files;

ALTER TABLE files_new RENAME TO files;

CREATE INDEX files_library_unavailable_scanned_idx
    ON files(library_id, unavailable_at, scanned_at);

PRAGMA foreign_keys = ON;
