DROP TRIGGER IF EXISTS node_metadata_search_fts_after_insert;
DROP TRIGGER IF EXISTS node_metadata_search_fts_after_update;
DROP TRIGGER IF EXISTS node_metadata_search_fts_after_delete;

DROP TABLE IF EXISTS node_search_fts;

CREATE VIRTUAL TABLE node_search_fts USING fts5(
    node_id UNINDEXED,
    node_metadata_id UNINDEXED,
    title,
    description,
    tokenize = 'unicode61 remove_diacritics 2'
);

INSERT INTO node_search_fts(rowid, node_id, node_metadata_id, title, description)
SELECT rowid, node_id, id, name, COALESCE(description, '')
FROM node_metadata;

CREATE TRIGGER node_metadata_search_fts_after_insert
AFTER INSERT ON node_metadata
BEGIN
    INSERT INTO node_search_fts(rowid, node_id, node_metadata_id, title, description)
    VALUES (new.rowid, new.node_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER node_metadata_search_fts_after_update
AFTER UPDATE ON node_metadata
BEGIN
    DELETE FROM node_search_fts WHERE rowid = old.rowid;
    INSERT INTO node_search_fts(rowid, node_id, node_metadata_id, title, description)
    VALUES (new.rowid, new.node_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER node_metadata_search_fts_after_delete
AFTER DELETE ON node_metadata
BEGIN
    DELETE FROM node_search_fts WHERE rowid = old.rowid;
END;
