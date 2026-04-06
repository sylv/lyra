DELETE FROM file_probe;

DROP TABLE file_probe;

CREATE TABLE file_probe (
    file_id TEXT PRIMARY KEY,
    probe BLOB NOT NULL,
    generated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
) STRICT;
