-- 0 poster, 1 thumbnail, 2 background, 3 timeline preview sheet
ALTER TABLE assets ADD COLUMN kind INTEGER NOT NULL DEFAULT 1;

UPDATE assets SET kind =
    CASE
        WHEN EXISTS (
            SELECT 1
            FROM file_assets
            WHERE file_assets.asset_id = assets.id
              AND file_assets.role = 0
        ) THEN 3
        WHEN EXISTS (
            SELECT 1
            FROM node_metadata
            WHERE node_metadata.poster_asset_id = assets.id
        ) THEN 0
        WHEN EXISTS (
            SELECT 1
            FROM node_metadata
            WHERE node_metadata.background_asset_id = assets.id
        ) THEN 2
        WHEN EXISTS (
            SELECT 1
            FROM node_metadata
            WHERE node_metadata.thumbnail_asset_id = assets.id
        ) THEN 1
        WHEN EXISTS (
            SELECT 1
            FROM file_assets
            WHERE file_assets.asset_id = assets.id
              AND file_assets.role = 1
        ) THEN 1
        -- Old rows can be orphaned or ambiguously shared because kind was not stored.
        ELSE 1
    END;
