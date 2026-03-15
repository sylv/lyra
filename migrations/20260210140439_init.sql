-- assets represent things like episode thumbnails, posters, backgrounds, etc.
CREATE TABLE assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
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

    -- if hash_sha256, then size_bytes, mime_type, height and width must be set
    CHECK (
        (hash_sha256 IS NOT NULL AND size_bytes IS NOT NULL AND mime_type IS NOT NULL AND height IS NOT NULL AND width IS NOT NULL) OR
        (hash_sha256 IS NULL)
    )
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
    last_scanned_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
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
    height INTEGER,
    width INTEGER,
    edition_name TEXT,
    audio_fingerprint BLOB NOT NULL,
    segments_json BLOB NOT NULL,
    keyframes_json BLOB NOT NULL,
    unavailable_at INTEGER,
    scanned_at INTEGER,
    discovered_at INTEGER NOT NULL DEFAULT (unixepoch()),

    UNIQUE (library_id, relative_path),
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file_assets (
    file_id INTEGER NOT NULL,
    asset_id INTEGER NOT NULL,
    role INTEGER NOT NULL,
    -- for chapter thumbnails
    chapter_number INTEGER,
    -- for frames taken from videos and sheets
    position_ms INTEGER,
    -- for sheets, where the sheet ends
    end_ms INTEGER,
    sheet_frame_height INTEGER,
    sheet_frame_width INTEGER,
    sheet_gap_size INTEGER,
    sheet_interval INTEGER,

    PRIMARY KEY (file_id, asset_id),
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file_probe (
    file_id INTEGER PRIMARY KEY,
    duration_s INTEGER,
    height INTEGER,
    width INTEGER,
    fps REAL,
    video_codec TEXT,
    video_bitrate INTEGER,
    audio_codec TEXT,
    audio_bitrate INTEGER,
    audio_channels INTEGER,
    has_subtitles INTEGER NOT NULL,
    -- ZSTD-compressed JSON from lyra-ffprobe (streams + format)
    streams BLOB,
    generated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    library_id INTEGER NOT NULL,
    root_id TEXT NOT NULL,
    parent_id TEXT,
    -- 0 movie, 1 series, 2 season, 3 episode
    kind INTEGER NOT NULL,
    name TEXT NOT NULL,
    "order" INTEGER NOT NULL,
    season_number INTEGER,
    episode_number INTEGER,
    match_candidates_json BLOB,
    last_added_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
    FOREIGN KEY (root_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES nodes(id) ON DELETE CASCADE,
    CHECK (kind IN (0, 1, 2, 3)),
    CHECK (
        (kind = 2 AND season_number IS NOT NULL AND episode_number IS NULL) OR
        (kind = 3 AND episode_number IS NOT NULL) OR
        (kind IN (0, 1) AND season_number IS NULL AND episode_number IS NULL)
    )
) STRICT;

-- closure is rebuilt transactionally by the scanner from the verified parent map.
CREATE TABLE node_closure (
    ancestor_id TEXT NOT NULL,
    descendant_id TEXT NOT NULL,
    depth INTEGER NOT NULL,

    PRIMARY KEY (ancestor_id, descendant_id),
    FOREIGN KEY (ancestor_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (descendant_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX node_closure_ancestor_depth_idx ON node_closure(ancestor_id, depth, descendant_id);
CREATE INDEX node_closure_descendant_depth_idx ON node_closure(descendant_id, depth, ancestor_id);

CREATE TABLE node_files (
    node_id TEXT NOT NULL,
    file_id INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    PRIMARY KEY (node_id, file_id),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX node_files_file_id_idx ON node_files(file_id);
CREATE INDEX node_files_node_order_idx ON node_files(node_id, "order", file_id);
-- order is a root-global traversal key used for playback and under-root listings.
CREATE UNIQUE INDEX nodes_root_order_idx ON nodes(root_id, "order");
CREATE UNIQUE INDEX nodes_parent_season_number_idx
    ON nodes(parent_id, season_number)
    WHERE kind = 2;
CREATE UNIQUE INDEX nodes_parent_episode_number_idx
    ON nodes(parent_id, episode_number)
    WHERE kind = 3;
CREATE INDEX nodes_library_root_idx ON nodes(library_id, root_id);
CREATE INDEX nodes_parent_order_idx ON nodes(parent_id, "order");

CREATE TABLE node_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    -- 0 local, 1 remote
    source INTEGER NOT NULL,
    provider_id TEXT NOT NULL,
    imdb_id TEXT,
    tmdb_id INTEGER,
    name TEXT NOT NULL,
    description TEXT,
    score_display TEXT,
    score_normalized INTEGER,
    released_at INTEGER,
    ended_at INTEGER,
    poster_asset_id INTEGER,
    thumbnail_asset_id INTEGER,
    background_asset_id INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (poster_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (thumbnail_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (background_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    CHECK (source IN (0, 1))
) STRICT;

CREATE UNIQUE INDEX node_metadata_unique_provider
    ON node_metadata(node_id, provider_id);
-- todo: when multiple remote providers exist, replace this with priority-based selection.
CREATE UNIQUE INDEX node_metadata_unique_source_layer
    ON node_metadata(node_id, source);
CREATE INDEX node_metadata_imdb_id_idx ON node_metadata(imdb_id);
CREATE INDEX node_metadata_tmdb_id_idx ON node_metadata(tmdb_id);

CREATE TABLE jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_kind INTEGER NOT NULL,
    subject_key TEXT NOT NULL UNIQUE,
    version_key INTEGER,
    file_id INTEGER,
    asset_id INTEGER,
    node_id TEXT,
    run_after INTEGER,
    last_run_at INTEGER NOT NULL,
    last_error_message TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE watch_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    file_id INTEGER NOT NULL,
    progress_percent REAL NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    UNIQUE (user_id, node_id)
) STRICT;
