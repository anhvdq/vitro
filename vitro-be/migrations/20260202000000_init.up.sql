CREATE EXTENSION IF NOT EXISTS pgcrypto; -- For using gen_random_uuid()

-- RBAC tables
CREATE TABLE workspaces (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text NULL,
  age integer NULL,
  email text NOT NULL UNIQUE,
  password_hash text NOT NULL,
  default_workspace_id uuid NULL REFERENCES workspaces(id) ON DELETE SET NULL, -- Default workspace is nullable
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_default_workspace ON users(default_workspace_id);

CREATE TABLE users_workspaces (
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  created_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, workspace_id)
);

CREATE INDEX idx_users_workspaces_workspace ON users_workspaces(workspace_id);

CREATE TABLE auth_identities (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  provider text NOT NULL,
  subject text NOT NULL,
  email text NULL,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (provider, subject)
);


-- Metadata tables

-- Create ENUM type
CREATE TYPE dataset_status AS ENUM ('ACTIVE', 'ARCHIVED');
CREATE TYPE asset_status AS ENUM ('ACTIVE', 'ARCHIVED');
CREATE TYPE task_type AS ENUM ('FULL_ANALYSIS');
CREATE TYPE task_status AS ENUM ('QUEUED', 'RUNNING', 'SUCCEEDED', 'FAILED', 'CANCELED');
CREATE TYPE task_run_status AS ENUM ('RUNNING', 'SUCCEEDED', 'FAILED', 'CANCELED');

CREATE TABLE datasets (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name text NOT NULL,
  display_name text NOT NULL,
  description text NULL,
  owner_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  tags text[] NOT NULL DEFAULT '{}', -- Default to empty text array
  status dataset_status NOT NULL DEFAULT 'ACTIVE',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  archived_at timestamptz NULL,
  UNIQUE (workspace_id, name) -- Name is snake_case & immutable, unique across workspace
);

CREATE INDEX idx_datasets_workspace ON datasets(workspace_id);
CREATE INDEX idx_datasets_workspace_status ON datasets(workspace_id, status);
CREATE INDEX idx_datasets_tags ON datasets USING GIN(tags);

CREATE TABLE assets (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE RESTRICT,
  name text NOT NULL,
  display_name text NOT NULL,
  description text NULL,
  owner_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  tags text[] NOT NULL DEFAULT '{}',
  connector_id uuid NOT NULL,
  connector_type text NOT NULL,
  storage_prefix text NOT NULL,
  allowed_formats text[] NOT NULL DEFAULT '{}', -- Default to empty text array
  status asset_status NOT NULL DEFAULT 'ACTIVE',
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  archived_at timestamptz NULL,
  UNIQUE (workspace_id, name)
);

CREATE INDEX idx_assets_workspace_dataset ON assets(workspace_id, dataset_id);
CREATE INDEX idx_assets_workspace_status ON assets(workspace_id, status);
CREATE INDEX idx_assets_tags ON assets USING GIN(tags);

CREATE TABLE tasks (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  workspace_id uuid NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE RESTRICT,
  asset_id uuid NOT NULL REFERENCES assets(id) ON DELETE RESTRICT,
  task_type task_type NOT NULL,
  status task_status NOT NULL DEFAULT 'QUEUED',
  configs jsonb NOT NULL DEFAULT '{}'::jsonb,
  correlation_id uuid NOT NULL,
  created_by uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_tasks_workspace_asset ON tasks(workspace_id, asset_id);
CREATE INDEX idx_tasks_workspace_status ON tasks(workspace_id, status);
CREATE INDEX idx_tasks_workspace_type ON tasks(workspace_id, task_type);

CREATE TABLE task_runs (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id uuid NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  attempt integer NOT NULL,
  status task_run_status NOT NULL,
  error_message text NULL,
  started_at timestamptz NULL,
  finished_at timestamptz NULL,
  metrics jsonb NOT NULL DEFAULT '{}'::jsonb,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (task_id, attempt)
);

CREATE INDEX idx_task_runs_task_created ON task_runs(task_id, created_at);
