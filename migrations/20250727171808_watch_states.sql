CREATE TABLE watch_state (
    media_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    progress_percentage REAL NOT NULL, -- 0-1
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL,

    PRIMARY KEY (media_id, user_id),
    FOREIGN KEY (media_id) REFERENCES media(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
) STRICT;