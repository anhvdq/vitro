CREATE TABLE files (
  id uuid PRIMARY KEY,
  asset_id uuid NOT NULL REFERENCES assets(id),
  workspace_id uuid NOT NULL REFERENCES workspaces(id),
  file_name text NOT NULL,
  storage_key text NOT NULL,
  size_bytes bigint NOT NULL,
  content_type text,
  uploaded_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_files_asset_id ON files(asset_id);
