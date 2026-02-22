-- assets represent things like episode thumbnails, posters, backgrounds, etc.
CREATE TABLE assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source INTEGER NOT NULL,
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

    -- if hash_sha256 or source == local, then size_bytes, mime_type, height and width must be set
    CHECK (
        (hash_sha256 IS NOT NULL AND source = 0 AND size_bytes IS NOT NULL AND mime_type IS NOT NULL AND height IS NOT NULL AND width IS NOT NULL) OR
        (hash_sha256 IS NULL OR source != 0)
    )
) STRICT;

-- require deleted_at be set to delete an asset, so that assets cannot be deleted without properly removing on-disk files
CREATE TRIGGER assets_prevent_delete_unless_marked
BEFORE DELETE ON assets
FOR EACH ROW
WHEN OLD.deleted_at IS NULL
BEGIN
  SELECT RAISE(ABORT, 'assets must be soft deleted (set deleted_at)');
END;

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
    unavailable_at INTEGER,
    corrupted_at INTEGER,
    scanned_at INTEGER,
    discovered_at INTEGER NOT NULL DEFAULT (unixepoch()),

    UNIQUE (library_id, relative_path),
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file_assets (
    file_id INTEGER NOT NULL,
    asset_id INTEGER NOT NULL,
    role INTEGER NOT NULL,
    chapter_number INTEGER, -- for chapter thumbnails
    position_ms INTEGER, -- for frames taken from videos and sheets
    end_ms INTEGER, -- for sheets, where the sheet ends
    sheet_frame_height INTEGER,
    sheet_frame_width INTEGER,
    sheet_gap_size INTEGER,
    sheet_interval INTEGER,

    PRIMARY KEY (file_id, asset_id),
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file_keyframes (
    file_id INTEGER PRIMARY KEY,
    keyframe_list BLOB NOT NULL, -- zstd-encoded json list of keyframe positions.
    generated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
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
    streams BLOB, -- ZSTD-compressed JSON, an array with info on each stream in the file
    generated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE roots (
    id TEXT PRIMARY KEY,
    library_id INTEGER NOT NULL,
    kind INTEGER NOT NULL, -- 0 movie, 1 series
    name TEXT NOT NULL,
    last_added_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
    CHECK (kind IN (0, 1))
) STRICT;

CREATE TABLE seasons (
    id TEXT PRIMARY KEY,
    root_id TEXT NOT NULL,
    season_number INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    name TEXT NOT NULL,
    last_added_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (root_id) REFERENCES roots(id) ON DELETE CASCADE,
    UNIQUE (root_id, season_number),
    UNIQUE (root_id, "order")
) STRICT;

CREATE TABLE items (
    id TEXT PRIMARY KEY,
    root_id TEXT NOT NULL,
    season_id TEXT,
    kind INTEGER NOT NULL, -- 0 movie, 1 episode
    episode_number INTEGER,
    "order" INTEGER NOT NULL,
    name TEXT NOT NULL,
    primary_file_id INTEGER,
    last_added_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (root_id) REFERENCES roots(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE SET NULL,
    FOREIGN KEY (primary_file_id) REFERENCES files(id) ON DELETE SET NULL,
    CHECK (kind IN (0, 1)),
    CHECK (
        (kind = 0 AND season_id IS NULL AND episode_number IS NULL) OR
        (kind = 1)
    ),
    UNIQUE (root_id, "order")
) STRICT;

CREATE TABLE item_files (
    item_id TEXT NOT NULL,
    file_id INTEGER NOT NULL,
    "order" INTEGER NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    PRIMARY KEY (item_id, file_id),
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX item_files_primary_unique ON item_files(item_id) WHERE is_primary = 1;
CREATE INDEX item_files_file_id_idx ON item_files(file_id);
CREATE INDEX item_files_item_order_idx ON item_files(item_id, "order", file_id);
CREATE INDEX items_root_order_idx ON items(root_id, "order");
CREATE INDEX items_season_order_idx ON items(season_id, "order");

CREATE TABLE root_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_id TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
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

    FOREIGN KEY (root_id) REFERENCES roots(id) ON DELETE CASCADE,
    FOREIGN KEY (poster_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (thumbnail_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (background_asset_id) REFERENCES assets(id) ON DELETE SET NULL
) STRICT;

CREATE UNIQUE INDEX root_metadata_unique_per_source
    ON root_metadata(source, source_key)
    WHERE source_key IS NOT NULL;
CREATE UNIQUE INDEX root_metadata_primary_unique
    ON root_metadata(root_id)
    WHERE is_primary = 1;

CREATE TABLE season_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    season_id TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
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

    FOREIGN KEY (season_id) REFERENCES seasons(id) ON DELETE CASCADE,
    FOREIGN KEY (poster_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (thumbnail_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (background_asset_id) REFERENCES assets(id) ON DELETE SET NULL
) STRICT;

CREATE UNIQUE INDEX season_metadata_unique_per_source
    ON season_metadata(source, source_key)
    WHERE source_key IS NOT NULL;
CREATE UNIQUE INDEX season_metadata_primary_unique
    ON season_metadata(season_id)
    WHERE is_primary = 1;

CREATE TABLE item_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id TEXT NOT NULL,
    source TEXT NOT NULL,
    source_key TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
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

    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
    FOREIGN KEY (poster_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (thumbnail_asset_id) REFERENCES assets(id) ON DELETE SET NULL,
    FOREIGN KEY (background_asset_id) REFERENCES assets(id) ON DELETE SET NULL
) STRICT;

CREATE UNIQUE INDEX item_metadata_unique_per_source
    ON item_metadata(source, source_key)
    WHERE source_key IS NOT NULL;
CREATE UNIQUE INDEX item_metadata_primary_unique
    ON item_metadata(item_id)
    WHERE is_primary = 1;

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- task_type + scope_kind + scope_id is the thing this task is meant to operate on, eg
    -- an asset, metadata, a file, etc.
    -- think of the two as the "unique key" for each job.
    task_type TEXT NOT NULL,
    scope_kind INTEGER NOT NULL,
    scope_id TEXT NOT NULL,
    -- zstd-compressed JSON blob, this is the data given to the task when it runs.
    input_args BLOB,
    -- if either version field changes, the task is re-run.
    -- version_number is a simple hard-coded value on each task type, when changed it forces all tasks of that type to re-run
    -- (ie, a bug was fixed in the task logic that caused incorrect results, so we want to re-run to fix it)
    -- version_hash is a optional hash of the relevant data that the task operates on, which when changed triggers a re-run
    -- (ie, intro detection runs on season-level data, but wants to re-run if episodes are added/removed)
    version_number INTEGER NOT NULL,
    version_hash TEXT,
    last_error_message TEXT,
    last_run_at INTEGER,
    execute_after INTEGER, -- if unset, never run this job. to run immediately, set to current time.
    locked_at INTEGER, -- when the job begins to run, this is set to stop other workers from running the same job.
    attempt_count INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE UNIQUE INDEX tasks_scope_unique ON tasks(task_type, scope_kind, scope_id);

CREATE TABLE watch_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    item_id TEXT NOT NULL,
    file_id INTEGER NOT NULL,
    progress_percent REAL NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES items(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    CHECK (progress_percent >= 0 AND progress_percent <= 1),
    UNIQUE (user_id, item_id)
) STRICT;

CREATE INDEX watch_progress_user_file_idx ON watch_progress(user_id, file_id);
