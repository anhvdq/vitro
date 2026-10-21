-- Event queue (docs/system-design.md "Event model"): events are kept for replay,
-- and each subscribed consumer gets one delivery row, written in the same
-- transaction as the state change that caused the event.
CREATE TABLE events (
  id uuid PRIMARY KEY,
  type text NOT NULL,
  version integer NOT NULL,
  tenant_id uuid NULL, -- workspace; NULL for platform-wide events
  occurred_at timestamptz NOT NULL,
  correlation_id uuid NOT NULL,
  causation_id uuid NULL,
  data jsonb NOT NULL -- references only, never personal data
);

CREATE INDEX idx_events_occurred_at ON events(occurred_at);

CREATE TABLE deliveries (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id uuid NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  consumer text NOT NULL,
  status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'done', 'dead')),
  attempts integer NOT NULL DEFAULT 0,
  max_attempts integer NOT NULL CHECK (max_attempts > 0),
  run_after timestamptz NOT NULL DEFAULT now(),
  locked_until timestamptz NULL,
  last_error text NULL,
  dedupe_key text NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Claim scan: due pending deliveries for one consumer.
CREATE INDEX idx_deliveries_claim ON deliveries(consumer, run_after) WHERE status = 'pending';

-- Debounce: at most one pending delivery per consumer and dedupe key.
CREATE UNIQUE INDEX uq_deliveries_pending_dedupe ON deliveries(consumer, dedupe_key)
  WHERE status = 'pending' AND dedupe_key IS NOT NULL;
