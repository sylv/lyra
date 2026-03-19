CREATE INDEX files_library_unavailable_scanned_idx
    ON files(library_id, unavailable_at, scanned_at);

CREATE INDEX nodes_root_kind_parent_numbers_idx
    ON nodes(root_id, kind, parent_id, season_number, episode_number, id);

CREATE INDEX node_metadata_node_source_updated_idx
    ON node_metadata(node_id, source, updated_at);
