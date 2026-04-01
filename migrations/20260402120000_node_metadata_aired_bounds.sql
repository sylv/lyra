ALTER TABLE node_metadata RENAME COLUMN released_at TO first_aired;
ALTER TABLE node_metadata RENAME COLUMN ended_at TO last_aired;

UPDATE node_metadata
SET last_aired = first_aired
WHERE first_aired IS NOT NULL
  AND last_aired IS NULL;

-- Existing series rows should sort sensibly before their metadata is resynced. Prefer the
-- most recent child air date from the same metadata layer, whether the series groups episodes
-- directly or through seasons.
UPDATE node_metadata AS series_metadata
SET last_aired = (
    SELECT MAX(COALESCE(child_metadata.last_aired, child_metadata.first_aired))
    FROM nodes AS child
    JOIN node_metadata AS child_metadata
        ON child_metadata.node_id = child.id
       AND child_metadata.source = series_metadata.source
    WHERE child.root_id = series_metadata.node_id
      AND child.id <> series_metadata.node_id
)
WHERE EXISTS (
    SELECT 1
    FROM nodes AS series
    WHERE series.id = series_metadata.node_id
      AND series.kind = 1
)
  AND EXISTS (
    SELECT 1
    FROM nodes AS child
    JOIN node_metadata AS child_metadata
        ON child_metadata.node_id = child.id
       AND child_metadata.source = series_metadata.source
    WHERE child.root_id = series_metadata.node_id
      AND child.id <> series_metadata.node_id
      AND COALESCE(child_metadata.last_aired, child_metadata.first_aired) IS NOT NULL
);
