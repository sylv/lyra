ALTER TABLE media ADD COLUMN first_linked_at INTEGER;

CREATE VIRTUAL TABLE search_index USING fts5(
    media_id,
    name,
    description,
    tokenize = "trigram"
);

CREATE TRIGGER media_insert_search_index AFTER INSERT ON media BEGIN
    INSERT INTO search_index(media_id, name, description)
    VALUES (NEW.id, NEW.name, NEW.description);
END;

CREATE TRIGGER media_update_search_index AFTER UPDATE ON media BEGIN
    UPDATE search_index 
    SET name = NEW.name, description = NEW.description
    WHERE media_id = NEW.id;
END;

CREATE TRIGGER media_delete_search_index AFTER DELETE ON media BEGIN
    DELETE FROM search_index WHERE media_id = OLD.id;
END;