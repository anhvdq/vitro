-- Server-side sessions: the client holds an opaque token, only its SHA-256 is stored,
-- so a leaked table cannot be replayed and a session can be revoked at any time.
CREATE TABLE sessions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash bytea NOT NULL UNIQUE,
  created_at timestamptz NOT NULL DEFAULT now(),
  expires_at timestamptz NOT NULL
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
