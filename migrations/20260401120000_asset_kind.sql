PRAGMA foreign_keys = OFF;

CREATE TABLE assets_new (
    id TEXT PRIMARY KEY,
    -- 0 poster, 1 thumbnail, 2 background, 3 timeline preview sheet
    kind INTEGER NOT NULL,
    -- when remote and not downloaded yet
    source_url TEXT,
    -- when stored locally
    hash_sha256 TEXT,
    size_bytes INTEGER,
    mime_type TEXT,
    height INTEGER,
    width INTEGER,
    thumbhash BLOB,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    CHECK (
        (hash_sha256 IS NOT NULL AND size_bytes IS NOT NULL AND mime_type IS NOT NULL AND height IS NOT NULL AND width IS NOT NULL) OR
        (hash_sha256 IS NULL)
    ),
    CHECK (kind IN (0, 1, 2, 3))
) STRICT;

INSERT INTO assets_new (
    id,
    kind,
    source_url,
    hash_sha256,
    size_bytes,
    mime_type,
    height,
    width,
    thumbhash,
    created_at
)
SELECT
    assets.id,
    CASE
        WHEN EXISTS (
            SELECT 1
            FROM file_assets
            WHERE file_assets.asset_id = assets.id
              AND file_assets.role = 0
        ) THEN 3
        WHEN EXISTS (
            SELECT 1
            FROM node_metadata
            WHERE node_metadata.poster_asset_id = assets.id
        ) THEN 0
        WHEN EXISTS (
            SELECT 1
            FROM node_metadata
            WHERE node_metadata.background_asset_id = assets.id
        ) THEN 2
        WHEN EXISTS (
            SELECT 1
            FROM node_metadata
            WHERE node_metadata.thumbnail_asset_id = assets.id
        ) THEN 1
        WHEN EXISTS (
            SELECT 1
            FROM file_assets
            WHERE file_assets.asset_id = assets.id
              AND file_assets.role = 1
        ) THEN 1
        -- Old rows can be orphaned or ambiguously shared because kind was not stored.
        ELSE 1
    END,
    assets.source_url,
    assets.hash_sha256,
    assets.size_bytes,
    assets.mime_type,
    assets.height,
    assets.width,
    assets.thumbhash,
    assets.created_at
FROM assets;

DROP TABLE assets;

ALTER TABLE assets_new RENAME TO assets;

PRAGMA foreign_keys = ON;
