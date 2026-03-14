CREATE VIRTUAL TABLE root_search_fts USING fts5(
    root_id UNINDEXED,
    root_metadata_id UNINDEXED,
    title,
    description,
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE VIRTUAL TABLE item_search_fts USING fts5(
    item_id UNINDEXED,
    item_metadata_id UNINDEXED,
    title,
    description,
    tokenize = 'unicode61 remove_diacritics 2'
);

INSERT INTO root_search_fts(rowid, root_id, root_metadata_id, title, description)
SELECT id, root_id, id, name, COALESCE(description, '')
FROM root_metadata;

INSERT INTO item_search_fts(rowid, item_id, item_metadata_id, title, description)
SELECT id, item_id, id, name, COALESCE(description, '')
FROM item_metadata;

CREATE TRIGGER root_metadata_search_fts_after_insert
AFTER INSERT ON root_metadata
BEGIN
    INSERT INTO root_search_fts(rowid, root_id, root_metadata_id, title, description)
    VALUES (new.id, new.root_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER root_metadata_search_fts_after_update
AFTER UPDATE ON root_metadata
BEGIN
    DELETE FROM root_search_fts WHERE rowid = old.id;
    INSERT INTO root_search_fts(rowid, root_id, root_metadata_id, title, description)
    VALUES (new.id, new.root_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER root_metadata_search_fts_after_delete
AFTER DELETE ON root_metadata
BEGIN
    DELETE FROM root_search_fts WHERE rowid = old.id;
END;

CREATE TRIGGER item_metadata_search_fts_after_insert
AFTER INSERT ON item_metadata
BEGIN
    INSERT INTO item_search_fts(rowid, item_id, item_metadata_id, title, description)
    VALUES (new.id, new.item_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER item_metadata_search_fts_after_update
AFTER UPDATE ON item_metadata
BEGIN
    DELETE FROM item_search_fts WHERE rowid = old.id;
    INSERT INTO item_search_fts(rowid, item_id, item_metadata_id, title, description)
    VALUES (new.id, new.item_id, new.id, new.name, COALESCE(new.description, ''));
END;

CREATE TRIGGER item_metadata_search_fts_after_delete
AFTER DELETE ON item_metadata
BEGIN
    DELETE FROM item_search_fts WHERE rowid = old.id;
END;
