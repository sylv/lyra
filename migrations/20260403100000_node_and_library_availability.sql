ALTER TABLE libraries ADD COLUMN unavailable_at INTEGER;

ALTER TABLE nodes ADD COLUMN unavailable_at INTEGER;

CREATE INDEX libraries_unavailable_at_idx ON libraries(unavailable_at);
CREATE INDEX nodes_library_unavailable_idx ON nodes(library_id, unavailable_at);
