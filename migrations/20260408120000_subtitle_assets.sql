ALTER TABLE assets ADD COLUMN asset_type INTEGER NOT NULL DEFAULT 0;
ALTER TABLE assets ADD COLUMN uncompressed_size_bytes INTEGER;
ALTER TABLE assets ADD COLUMN content_encoding TEXT;

-- existing assets are all images
UPDATE assets SET asset_type = 0;

PRAGMA foreign_keys = OFF;

CREATE TABLE assets_new (
    id TEXT PRIMARY KEY,
    kind INTEGER NOT NULL,
    asset_type INTEGER NOT NULL,
    source_url TEXT,
    hash_sha256 TEXT,
    size_bytes INTEGER,
    uncompressed_size_bytes INTEGER,
    mime_type TEXT,
    content_encoding TEXT,
    height INTEGER,
    width INTEGER,
    thumbhash BLOB,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;

INSERT INTO assets_new (
    id,
    kind,
    asset_type,
    source_url,
    hash_sha256,
    size_bytes,
    uncompressed_size_bytes,
    mime_type,
    content_encoding,
    height,
    width,
    thumbhash,
    created_at
)
SELECT
    id,
    kind,
    asset_type,
    source_url,
    hash_sha256,
    size_bytes,
    uncompressed_size_bytes,
    mime_type,
    content_encoding,
    height,
    width,
    thumbhash,
    created_at
FROM assets;

DROP TABLE assets;

ALTER TABLE assets_new RENAME TO assets;

PRAGMA foreign_keys = ON;

ALTER TABLE files ADD COLUMN subtitles_extracted_at INTEGER;

CREATE TABLE file_subtitles (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    derived_from_subtitle_id TEXT,
    kind INTEGER NOT NULL,
    source INTEGER NOT NULL,
    stream_index INTEGER NOT NULL,
    language_bcp47 TEXT,
    display_name TEXT,
    disposition_bits INTEGER NOT NULL DEFAULT 0,
    last_seen_at INTEGER NOT NULL,
    processed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY (derived_from_subtitle_id) REFERENCES file_subtitles(id) ON DELETE SET NULL
) STRICT;

CREATE INDEX file_subtitles_file_stream_idx
    ON file_subtitles(file_id, stream_index);
CREATE INDEX file_subtitles_file_processed_idx
    ON file_subtitles(file_id, processed_at);
CREATE INDEX file_subtitles_file_last_seen_idx
    ON file_subtitles(file_id, last_seen_at);
CREATE INDEX file_subtitles_parent_idx
    ON file_subtitles(derived_from_subtitle_id);
CREATE UNIQUE INDEX file_subtitles_unique_extracted_source_idx
    ON file_subtitles(file_id, stream_index, source)
    WHERE derived_from_subtitle_id IS NULL;
CREATE UNIQUE INDEX file_subtitles_unique_derived_variant_idx
    ON file_subtitles(derived_from_subtitle_id, kind, source)
    WHERE derived_from_subtitle_id IS NOT NULL;
