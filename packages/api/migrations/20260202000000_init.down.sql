-- Drop metadata tables
DROP TABLE task_runs;
DROP TABLE tasks;
DROP TABLE assets;
DROP TABLE datasets;

-- Drop RBAC tables
DROP TABLE auth_identities;
DROP TABLE users_workspaces;
DROP TABLE users;
DROP TABLE workspaces;

-- Drop enum types
DROP TYPE task_run_status;
DROP TYPE task_status;
DROP TYPE task_type;
DROP TYPE asset_status;
DROP TYPE dataset_status;

-- Note: we do NOT drop the `pgcrypto` extension as it might be used somewhere else in the database.
