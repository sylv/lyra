CREATE TABLE images (
    id TEXT PRIMARY KEY,
    kind INTEGER NOT NULL,
    source INTEGER NOT NULL,
    -- when remote and not downloaded yet
    source_url TEXT,
    size_bytes INTEGER,
    mime_type TEXT,
    height INTEGER,
    width INTEGER,
    thumbhash BLOB,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER
) STRICT;

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    invite_code TEXT,
    permissions INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    preferred_audio_language TEXT,
    preferred_audio_disposition TEXT,
    -- 0 off, 1 auto, 2 on
    subtitle_mode INTEGER NOT NULL DEFAULT 1,
    preferred_subtitle_languages TEXT NOT NULL DEFAULT '[]',
    -- 0 any, 1 forced, 2 normal, 3 sdh, 4 commentary
    subtitle_variant_preference INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    user_agent TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    last_seen_at INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE libraries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    last_scanned_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    pinned INTEGER NOT NULL DEFAULT 1,
    unavailable_at INTEGER
) STRICT;

CREATE TABLE files (
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
    subtitles_extracted_at INTEGER,

    UNIQUE (library_id, relative_path),
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file_images (
    file_id TEXT NOT NULL,
    image_id TEXT NOT NULL,
    role INTEGER NOT NULL,
    start_ms INTEGER, -- for sheets, where the sheet starts
    end_ms INTEGER, -- for sheets, where the sheet ends (exclusive)
    sheet_frame_height INTEGER,
    sheet_frame_width INTEGER,
    sheet_gap_size INTEGER,
    sheet_interval INTEGER,

    PRIMARY KEY (file_id, image_id),
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file_probe (
    file_id TEXT PRIMARY KEY,
    -- zstd-compressed JSON from lyra-ffprobe (streams + format)
    probe BLOB NOT NULL,
    extracted_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file_subtitles (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    source_stream_index INTEGER NOT NULL,
    language_bcp47 TEXT,
    display_name TEXT,
    disposition_bits INTEGER NOT NULL DEFAULT 0,
    extracted_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    library_id TEXT NOT NULL,
    root_id TEXT NOT NULL,
    parent_id TEXT,
    -- 0 movie, 1 series, 2 season, 3 episode
    kind INTEGER NOT NULL,
    name TEXT NOT NULL,
    "order" INTEGER NOT NULL,
    season_number INTEGER,
    episode_number INTEGER,
    last_added_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    unavailable_at INTEGER,
    last_fingerprint_version INTEGER,

    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
    FOREIGN KEY (root_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX nodes_order ON nodes(root_id, "order");

CREATE VIRTUAL TABLE node_closure USING transitive_closure(
    tablename='nodes',
    idcolumn='id',
    parentcolumn='parent_id'
);

CREATE TABLE node_files (
    node_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    "order" INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    PRIMARY KEY (node_id, file_id),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX node_files_file_id_idx ON node_files(file_id);
CREATE INDEX node_files_node_order_idx ON node_files(node_id, "order", file_id);

CREATE TABLE node_metadata (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    source INTEGER NOT NULL,
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
    content_ratings BLOB, -- zstd json array of {country_code, rating, release_date, release_type}
    next_aired INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX node_metadata_unique_source ON node_metadata(node_id, source);

CREATE TABLE node_images (
    id TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    image_id TEXT NOT NULL,
    source INTEGER NOT NULL,
    kind INTEGER NOT NULL,
    language TEXT,
    vote_average REAL,
    vote_count INTEGER,
    width INTEGER,
    height INTEGER,
    file_type TEXT,
    is_active INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
) STRICT;

CREATE VIRTUAL TABLE node_search_fts USING fts5(
    node_id UNINDEXED,
    node_metadata_id UNINDEXED,
    title,
    description,
    tokenize = 'unicode61 remove_diacritics 2'
);

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

    UNIQUE (job_kind, target_id)
) STRICT;

CREATE TABLE watch_progress (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    tmdb_id INTEGER,
    file_id TEXT NOT NULL,
    progress_percent REAL NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    UNIQUE (user_id, node_id)
) STRICT;

CREATE VIEW image_references AS
SELECT image_id, 'node_image' AS ref_kind, node_id AS ref_id FROM node_images
UNION ALL
SELECT image_id, 'file_image' AS ref_kind, file_id AS ref_id FROM file_images;
