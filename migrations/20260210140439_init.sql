-- assets represent things like episode thumbnails, posters, backgrounds, etc.
CREATE TABLE assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind INTEGER NOT NULL,
    provider TEXT NOT NULL,
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
    deleted_at INTEGER,

    CHECK ((hash_sha256 IS NOT NULL AND source_url IS NULL) OR (hash_sha256 IS NULL AND source_url IS NOT NULL))
) STRICT;

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    invite_code TEXT,
    permissions INTEGER NOT NULL,
    default_subtitle_iso639_1 TEXT,
    default_audio_iso639_1 TEXT,
    subtitles_enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    CHECK (
        (password_hash IS NOT NULL AND invite_code IS NULL) OR
        (password_hash IS NULL AND invite_code IS NOT NULL)
    )
) STRICT;

CREATE TABLE user_sessions (
    token TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE libraries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    last_scanned_at INTEGER
) STRICT;

CREATE TABLE library_users (
    library_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,

    PRIMARY KEY (library_id, user_id),
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    relative_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    hash_5mb_sha256 TEXT,
    duration_s INTEGER NOT NULL DEFAULT 0,
    height INTEGER,
    width INTEGER,
    edition_name TEXT,
    unavailable_at INTEGER,
    corrupted_at INTEGER,
    scanned_at INTEGER,
    discovered_at INTEGER NOT NULL DEFAULT (unixepoch()),

    UNIQUE (library_id, relative_path),
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
) STRICT;

-- nodes are derived from file paths and parser output.
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    root_id TEXT,
    parent_id TEXT,
    library_id INTEGER NOT NULL,
    file_id INTEGER,
    relative_path TEXT NOT NULL,
    name TEXT NOT NULL,
    kind INTEGER NOT NULL, -- 0 movie, 1 series, 2 season, 3 episode

    FOREIGN KEY (root_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE SET NULL,
    CHECK (
        (parent_id IS NULL AND root_id IS NULL AND kind IN (0, 1)) OR
        (parent_id IS NOT NULL AND root_id IS NOT NULL AND kind IN (2, 3))
    )
) STRICT;

CREATE TABLE node_metadata (
    node_id TEXT NOT NULL,
    metadata_id INTEGER NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,

    PRIMARY KEY (node_id, metadata_id),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (metadata_id) REFERENCES metadata(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX node_metadata_primary_unique ON node_metadata(node_id) WHERE is_primary = 1;

-- this stores information about providers attempting to match a node 
CREATE TABLE node_matches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    source TEXT NOT NULL,
    metadata_id INTEGER,
    error_reason TEXT,
    attempted_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (metadata_id) REFERENCES metadata(id) ON DELETE SET NULL
) STRICT;

-- metadata layers from providers (local, tmdb, ...)
CREATE TABLE metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_id INTEGER,
    parent_id INTEGER,
    source TEXT NOT NULL,
    source_key TEXT,
    kind INTEGER NOT NULL, -- 0 movie, 1 series, 2 season, 3 episode
    name TEXT NOT NULL,
    description TEXT,
    score_display TEXT,
    score_normalized INTEGER,
    season_number INTEGER,
    episode_number INTEGER,
    released_at INTEGER,
    ended_at INTEGER,
    poster_asset_id INTEGER,
    thumbnail_asset_id INTEGER,
    background_asset_id INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (root_id) REFERENCES metadata(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES metadata(id) ON DELETE SET NULL,
    FOREIGN KEY (poster_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (thumbnail_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (background_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    CHECK (
        (parent_id IS NULL AND root_id IS NULL AND kind IN (0, 1)) OR
        (parent_id IS NOT NULL AND root_id IS NOT NULL AND kind IN (2, 3))
    )
) STRICT;

CREATE UNIQUE INDEX metadata_unique_per_source ON metadata(source, source_key) WHERE source_key IS NOT NULL;

-- playback progress stores both node + concrete file when known
CREATE TABLE watch_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    file_id INTEGER,
    node_id TEXT,
    progress_percent REAL NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE SET NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE SET NULL,
    CHECK (progress_percent >= 0 AND progress_percent <= 1)
) STRICT;

CREATE UNIQUE INDEX watch_progress_user_file_unique
    ON watch_progress(user_id, file_id)
    WHERE file_id IS NOT NULL;

CREATE UNIQUE INDEX watch_progress_user_node_unique
    ON watch_progress(user_id, node_id)
    WHERE node_id IS NOT NULL;

CREATE VIRTUAL TABLE metadata_fts5 USING fts5(
    metadata_id UNINDEXED,
    name,
    tokenize = 'trigram'
);

INSERT INTO metadata_fts5 (metadata_id, name)
SELECT id, name FROM metadata;

CREATE TRIGGER metadata_insert_fts5 AFTER INSERT ON metadata BEGIN
    INSERT INTO metadata_fts5 (metadata_id, name)
    VALUES (NEW.id, NEW.name);
END;

CREATE TRIGGER metadata_update_fts5 AFTER UPDATE ON metadata BEGIN
    UPDATE metadata_fts5
    SET name = NEW.name
    WHERE metadata_id = NEW.id;
END;

CREATE TRIGGER metadata_delete_fts5 AFTER DELETE ON metadata BEGIN
    DELETE FROM metadata_fts5 WHERE metadata_id = OLD.id;
END;
