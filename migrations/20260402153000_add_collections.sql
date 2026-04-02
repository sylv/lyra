ALTER TABLE libraries ADD COLUMN pinned INTEGER NOT NULL DEFAULT 1;

CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_by_id TEXT,
    -- 0 public, 1 private
    visibility INTEGER NOT NULL,
    -- 0 manual, 1 filter
    resolver_kind INTEGER NOT NULL,
    -- 0 continue watching
    kind INTEGER,
    filter_json BLOB,
    show_on_home INTEGER NOT NULL DEFAULT 0,
    home_position INTEGER NOT NULL DEFAULT 0,
    pinned INTEGER NOT NULL DEFAULT 0,
    pinned_position INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    FOREIGN KEY (created_by_id) REFERENCES users(id) ON DELETE CASCADE,
    CHECK (visibility IN (0, 1)),
    CHECK (resolver_kind IN (0, 1)),
    CHECK (kind IN (0) OR kind IS NULL)
) STRICT;

CREATE UNIQUE INDEX collections_kind_idx
    ON collections(kind)
    WHERE kind IS NOT NULL;
CREATE INDEX collections_home_idx
    ON collections(show_on_home, home_position, created_at);
CREATE INDEX collections_pinned_idx
    ON collections(pinned, pinned_position, created_at);
CREATE INDEX collections_created_by_idx
    ON collections(created_by_id, visibility, created_at);

CREATE TABLE collection_items (
    collection_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),

    PRIMARY KEY (collection_id, node_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT;

CREATE UNIQUE INDEX collection_items_position_idx
    ON collection_items(collection_id, position);
CREATE INDEX collection_items_node_id_idx
    ON collection_items(node_id);
