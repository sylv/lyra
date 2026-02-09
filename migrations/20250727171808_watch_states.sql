CREATE TABLE watch_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    media_id INTEGER NOT NULL,
    progress_percentage REAL NOT NULL, -- 0-1
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL,

    UNIQUE (user_id, media_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (media_id) REFERENCES media(id) ON DELETE CASCADE,
    CHECK (progress_percentage >= 0 AND progress_percentage <= 1)
) STRICT;