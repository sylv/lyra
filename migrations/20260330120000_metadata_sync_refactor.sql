ALTER TABLE nodes DROP COLUMN match_candidates_json;

DELETE FROM jobs
WHERE job_kind IN (7, 8);
