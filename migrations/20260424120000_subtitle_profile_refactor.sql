ALTER TABLE users ADD COLUMN subtitle_mode INTEGER NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN preferred_subtitle_languages TEXT NOT NULL DEFAULT '[]';
ALTER TABLE users ADD COLUMN subtitle_variant_preference INTEGER NOT NULL DEFAULT 0;

UPDATE users
SET preferred_subtitle_languages = json_array(preferred_subtitle_language)
WHERE preferred_subtitle_language IS NOT NULL AND preferred_subtitle_language != '';

UPDATE users
SET subtitle_variant_preference = CASE preferred_subtitle_disposition
    WHEN 'Normal' THEN 2
    WHEN 'Sdh' THEN 3
    WHEN 'Commentary' THEN 4
    ELSE 0
END;

ALTER TABLE users DROP COLUMN preferred_subtitle_language;
ALTER TABLE users DROP COLUMN preferred_subtitle_disposition;
