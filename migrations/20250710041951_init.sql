-- "media" represents a movie, show, season or episode.
-- it is always either a collection of child media or a file.
-- for a show, the hierarchy would be Media(The Expanse) -> Media(Season 1) -> Media(Episode 1) -> File(Episode 1.mp4)
CREATE TABLE media (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    poster_url TEXT,
    background_url TEXT,
    thumbnail_url TEXT,
    parent_id INTEGER,
    media_type INTEGER NOT NULL,
    tmdb_parent_id INTEGER NOT NULL, -- show or movie id
    tmdb_item_id INTEGER, -- episode id
    rating REAL, -- 0-10/10
    release_date INTEGER, -- unix timestamp
    runtime_minutes INTEGER,
    season_number INTEGER,
    episode_number INTEGER,

    FOREIGN KEY (parent_id) REFERENCES media(id),
    CHECK (media_type IN (0, 1, 2, 3)) -- check media_type is valid and will trigger the unique constraints
) STRICT;

-- if media_type is 0 (movie) or 1 (show), ensure tmdb_parent_id is unique
CREATE UNIQUE INDEX media_unique_tmdb_parent_id ON media (tmdb_parent_id) WHERE media_type IN (0, 1);
-- if media_type is 2 (season), ensure tmdb_parent_id and season_number are unique
CREATE UNIQUE INDEX media_unique_tmdb_parent_id_season_number ON media (tmdb_parent_id, season_number) WHERE media_type = 2;
-- if media_type is 3 (episode), ensure tmdb_parent_id, season_number and episode_number are unique
CREATE UNIQUE INDEX media_unique_tmdb_parent_id_season_number_episode_number ON media (tmdb_parent_id, season_number, episode_number) WHERE media_type = 3;

-- connects a File to a Media item.
-- a single file can be connected to multiple media items (eg, a movie can have multiple editions) 
-- and a single file to multiple media items (eg, a multi-episode file)
CREATE TABLE media_connection (
    media_id INTEGER NOT NULL,
    file_id INTEGER NOT NULL,

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
    scanned_at INTEGER NOT NULL
) STRICT;

CREATE UNIQUE INDEX file_backend_key_unique ON file (backend_name, key); 
