-- Create webhooks table for space event notifications
CREATE TABLE IF NOT EXISTS space_webhooks (
    id UUID PRIMARY KEY NOT NULL,
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_space_webhooks_space_id ON space_webhooks(space_id);
