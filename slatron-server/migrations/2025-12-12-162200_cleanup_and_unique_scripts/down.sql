-- Drop the unique index
DROP INDEX idx_scripts_name_unique;

-- (Data is lost, cannot restore in down migration)
