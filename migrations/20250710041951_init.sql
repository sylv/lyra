-- "media" represents a movie, show or episode.
-- for example: Media(The Expanse) -> Media(The Expanse S01E01)
CREATE TABLE media (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    imdb_id TEXT UNIQUE,
    tmdb_id INTEGER UNIQUE,
    parent_id INTEGER, -- for episodes of shows
    kind INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    rating REAL, -- 0-10/10
    poster_url TEXT,
    background_url TEXT,
    thumbnail_url TEXT,
    season_number INTEGER,
    episode_number INTEGER,
    runtime_minutes INTEGER,
    released_at INTEGER,
    ended_at INTEGER, -- only set for tv shows that have ended
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER,
    
    CHECK (kind IN (0, 1, 2)),
    CHECK (kind != 2 OR (parent_id IS NOT NULL AND season_number IS NOT NULL AND episode_number IS NOT NULL)),
    CHECK (kind = 2 OR parent_id IS NULL)
) STRICT;

-- this is mostly used as a target for tasks.
CREATE TABLE season (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    media_id INTEGER NOT NULL,
    season_number INTEGER NOT NULL,

    UNIQUE (media_id, season_number),
    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE
) STRICT;

-- connects a File to a Media item.
-- a single file can be connected to multiple media items (eg, a multi-episode file)
CREATE TABLE media_connection (
    media_id INTEGER NOT NULL,
    file_id INTEGER NOT NULL,

    PRIMARY KEY (media_id, file_id),
    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES file(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE file (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    relative_path TEXT NOT NULL,
    pending_auto_match INTEGER NOT NULL DEFAULT 1,
    edition_name TEXT, -- eg, "Director's Cut" or null, parsed from the file name (eg, "{edition=Director's Cut}")
    resolution INTEGER, -- eg, 1080 or 2160, parsed from the file name
    size_bytes INTEGER,
    scanned_at INTEGER NOT NULL,
    unavailable_at INTEGER,
    corrupted_at INTEGER,

    UNIQUE (library_id, relative_path),
    FOREIGN KEY (library_id) REFERENCES library(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE library (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    last_scanned_at INTEGER
) STRICT;

CREATE TABLE library_user (
    library_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,

    PRIMARY KEY (library_id, user_id),
    FOREIGN KEY (library_id) REFERENCES library(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    invite_code TEXT,
    permissions INTEGER NOT NULL,
    default_subtitle_bcp47 TEXT, -- eg "en-US"
    default_audio_bcp47 TEXT,
    subtitles_enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),

    -- ensure either password or invite code is set, but not both
    CHECK (
        (password_hash IS NOT NULL AND invite_code IS NULL) OR 
        (password_hash IS NULL AND invite_code IS NOT NULL)
    )
) STRICT;

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) STRICT;