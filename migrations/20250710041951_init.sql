-- "media" represents a movie, show, or episode.
-- it is always either a collection of child media or a file.
-- for a show, the hierarchy would be Media(The Expanse) -> Media(Episode 1) -> File(Episode 1.mp4)
CREATE TABLE media (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    poster_url TEXT,
    background_url TEXT,
    thumbnail_url TEXT,
    parent_id INTEGER, -- for episodes, this is the show id
    media_type INTEGER NOT NULL,
    imdb_parent_id TEXT,
    imdb_item_id TEXT,
    tmdb_parent_id INTEGER NOT NULL, -- show or movie id
    tmdb_item_id INTEGER NOT NULL, -- episode id (for episodes) or show id (for shows, mostly for unique constraint)
    rating REAL, -- 0-10/10
    start_date INTEGER, -- unix timestamp, aka release_date for movies
    end_date INTEGER, -- unix timestamp, null for movies
    runtime_minutes INTEGER,
    season_number INTEGER,
    episode_number INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER,

    FOREIGN KEY (parent_id) REFERENCES media(id),
    UNIQUE (tmdb_parent_id, tmdb_item_id),
    CHECK (media_type IN (0, 1, 2)) -- check media_type is valid and will trigger the unique constraints
) STRICT;

-- connects a File to a Media item.
-- a single file can be connected to multiple media items (eg, a movie can have multiple editions) 
-- and a single file to multiple media items (eg, a multi-episode file)
CREATE TABLE media_connection (
    media_id INTEGER NOT NULL,
    file_id INTEGER NOT NULL,

    PRIMARY KEY (media_id, file_id),
    FOREIGN KEY (media_id) REFERENCES media(id),
    FOREIGN KEY (file_id) REFERENCES file(id)
) STRICT;

CREATE TABLE file (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    backend_name TEXT NOT NULL, -- name of the backend, eg "vault" that maps to a webdav store
    key TEXT NOT NULL, -- generally the path fo the file on the backend
    pending_auto_match INTEGER NOT NULL DEFAULT 1,
    unavailable_since INTEGER,
    edition_name TEXT, -- eg, "Director's Cut" or null, parsed from the file name (eg, "{edition=Director's Cut}")
    resolution INTEGER, -- eg, 1080 or 2160, parsed from the file name
    size_bytes INTEGER,
    scanned_at INTEGER NOT NULL,

    UNIQUE (backend_name, key)
) STRICT;

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    permissions INTEGER NOT NULL,
    default_subtitle_bcp47 TEXT, -- eg "en-US"
    default_audio_bcp47 TEXT,
    subtitles_enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    last_login_at INTEGER
) STRICT;

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id)
) STRICT;

CREATE TABLE invites (
    code TEXT PRIMARY KEY,
    permissions INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER NOT NULL,
    used_at INTEGER,
    used_by TEXT,

    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (used_by) REFERENCES users(id)
) STRICT;