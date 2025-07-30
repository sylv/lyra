ALTER TABLE media ADD COLUMN first_linked_at INTEGER;

CREATE VIRTUAL TABLE media_fts5 USING fts5(
    id,
    name,
    description,
    tokenize = "trigram"
);

CREATE TRIGGER media_fts5_insert AFTER INSERT ON media BEGIN
    INSERT INTO media_fts5(id, name, description)
    VALUES (NEW.id, NEW.name, NEW.description);
END;

CREATE TRIGGER media_fts5_update AFTER UPDATE ON media BEGIN
    UPDATE media_fts5 
    SET name = NEW.name, description = NEW.description
    WHERE id = NEW.id;
END;

CREATE TRIGGER media_fts5_delete AFTER DELETE ON media BEGIN
    DELETE FROM media_fts5 WHERE id = OLD.id;
END;
