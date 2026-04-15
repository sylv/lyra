DELETE FROM jobs
WHERE job_kind = 7;

DELETE FROM node_metadata
WHERE source = 1;

DROP TRIGGER IF EXISTS node_metadata_search_fts_after_insert;
DROP TRIGGER IF EXISTS node_metadata_search_fts_after_update;
DROP TRIGGER IF EXISTS node_metadata_search_fts_after_delete;
DROP TABLE IF EXISTS node_search_fts;
DROP VIEW IF EXISTS asset_references;

PRAGMA foreign_keys = OFF;

CREATE TABLE node_metadata_new (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    source INTEGER NOT NULL,
    provider_id TEXT NOT NULL,
    imdb_id TEXT,
    tmdb_id INTEGER,
    name TEXT NOT NULL,
    description TEXT,
    score_display TEXT,
    score_normalized INTEGER,
    first_aired INTEGER,
    last_aired INTEGER,
    status INTEGER,
    tagline TEXT,
    next_aired INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    CHECK (source IN (0, 1))
) STRICT;

INSERT INTO node_metadata_new (
    id,
    node_id,
    source,
    provider_id,
    imdb_id,
    tmdb_id,
    name,
    description,
    score_display,
    score_normalized,
    first_aired,
    last_aired,
    status,
    tagline,
    next_aired,
    created_at,
    updated_at
)
SELECT
    id,
    node_id,
    source,
    provider_id,
    imdb_id,
    tmdb_id,
    name,
    description,
    score_display,
    score_normalized,
    first_aired,
    last_aired,
    NULL,
    NULL,
    NULL,
    created_at,
    updated_at
FROM node_metadata;

CREATE TABLE node_metadata_images (
    id TEXT PRIMARY KEY,
    node_metadata_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    kind INTEGER NOT NULL,
    position INTEGER NOT NULL,
    language TEXT,
    vote_average REAL,
    vote_count INTEGER,
    width INTEGER,
    height INTEGER,
    file_type TEXT,
    is_active INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_metadata_id) REFERENCES node_metadata_new(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX node_metadata_images_active_kind_idx
    ON node_metadata_images(node_metadata_id, kind)
    WHERE is_active = 1;
CREATE UNIQUE INDEX node_metadata_images_position_idx
    ON node_metadata_images(node_metadata_id, kind, position);
CREATE INDEX node_metadata_images_asset_id_idx
    ON node_metadata_images(asset_id);
CREATE INDEX node_metadata_images_metadata_kind_idx
    ON node_metadata_images(node_metadata_id, kind, is_active, position);

CREATE TABLE node_metadata_recommendations (
    id TEXT PRIMARY KEY,
    node_metadata_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    media_kind INTEGER NOT NULL,
    tmdb_id INTEGER,
    imdb_id TEXT,
    name TEXT NOT NULL,
    first_aired INTEGER,
    position INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_metadata_id) REFERENCES node_metadata_new(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX node_metadata_recommendations_position_idx
    ON node_metadata_recommendations(node_metadata_id, position);
CREATE INDEX node_metadata_recommendations_lookup_idx
    ON node_metadata_recommendations(provider_id, media_kind, tmdb_id, imdb_id);

CREATE TABLE node_metadata_genres (
    id TEXT PRIMARY KEY,
    node_metadata_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    external_id TEXT,
    name TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_metadata_id) REFERENCES node_metadata_new(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX node_metadata_genres_position_idx
    ON node_metadata_genres(node_metadata_id, position);

CREATE TABLE node_metadata_content_ratings (
    id TEXT PRIMARY KEY,
    node_metadata_id TEXT NOT NULL,
    country_code TEXT NOT NULL,
    rating TEXT NOT NULL,
    release_date INTEGER,
    release_type INTEGER,
    position INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_metadata_id) REFERENCES node_metadata_new(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX node_metadata_content_ratings_position_idx
    ON node_metadata_content_ratings(node_metadata_id, country_code, position);
CREATE INDEX node_metadata_content_ratings_country_idx
    ON node_metadata_content_ratings(country_code, rating);

DROP TABLE node_metadata;
ALTER TABLE node_metadata_new RENAME TO node_metadata;

CREATE UNIQUE INDEX node_metadata_unique_provider
    ON node_metadata(node_id, provider_id);
CREATE UNIQUE INDEX node_metadata_unique_source_layer
    ON node_metadata(node_id, source);
CREATE INDEX node_metadata_imdb_id_idx ON node_metadata(imdb_id);
CREATE INDEX node_metadata_tmdb_id_idx ON node_metadata(tmdb_id);
CREATE INDEX node_metadata_node_source_updated_idx
    ON node_metadata(node_id, source, updated_at);

CREATE VIRTUAL TABLE node_search_fts USING fts5(
    node_id UNINDEXED,
    node_metadata_id UNINDEXED,
    title,
    description,
    tokenize = 'unicode61 remove_diacritics 2'
);

INSERT INTO node_search_fts(rowid, node_id, node_metadata_id, title, description)
SELECT rowid, node_id, id, name, COALESCE(description, '')
FROM node_metadata;

CREATE TRIGGER node_metadata_search_fts_after_insert
AFTER INSERT ON node_metadata
BEGIN
    INSERT INTO node_search_fts(rowid, node_id, node_metadata_id, title, description)
    VALUES (new.rowid, new.node_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER node_metadata_search_fts_after_update
AFTER UPDATE ON node_metadata
BEGIN
    DELETE FROM node_search_fts WHERE rowid = old.rowid;
    INSERT INTO node_search_fts(rowid, node_id, node_metadata_id, title, description)
    VALUES (new.rowid, new.node_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER node_metadata_search_fts_after_delete
AFTER DELETE ON node_metadata
BEGIN
    DELETE FROM node_search_fts WHERE rowid = old.rowid;
END;

ALTER TABLE assets ADD COLUMN updated_at INTEGER;

UPDATE assets
SET updated_at = created_at
WHERE updated_at IS NULL;

CREATE TABLE people (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    provider_person_id TEXT NOT NULL,
    name TEXT NOT NULL,
    birthday TEXT,
    description TEXT,
    profile_asset_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (profile_asset_id) REFERENCES assets(id) ON DELETE SET NULL
) STRICT;

CREATE UNIQUE INDEX people_unique_provider_person_idx
    ON people(provider_id, provider_person_id);
CREATE INDEX people_profile_asset_idx
    ON people(profile_asset_id);

CREATE TABLE root_node_cast (
    id TEXT PRIMARY KEY,
    root_node_id TEXT NOT NULL,
    person_id TEXT NOT NULL,
    character_name TEXT,
    department TEXT,
    position INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (root_node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES people(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX root_node_cast_root_position_idx
    ON root_node_cast(root_node_id, position);
CREATE UNIQUE INDEX root_node_cast_root_person_character_idx
    ON root_node_cast(root_node_id, person_id, character_name);
CREATE INDEX root_node_cast_person_idx
    ON root_node_cast(person_id, root_node_id);

CREATE VIEW asset_references AS
SELECT asset_id, 'node_metadata_image' AS ref_kind, node_metadata_id AS ref_id
FROM node_metadata_images
UNION ALL
SELECT asset_id, 'file_asset' AS ref_kind, file_id AS ref_id
FROM file_assets
UNION ALL
SELECT asset_id, 'file_subtitle' AS ref_kind, file_id AS ref_id
FROM file_subtitles
UNION ALL
SELECT profile_asset_id AS asset_id, 'person_profile' AS ref_kind, id AS ref_id
FROM people
WHERE profile_asset_id IS NOT NULL;

PRAGMA foreign_keys = ON;
