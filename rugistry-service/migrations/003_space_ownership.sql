-- Add owner_id to spaces (nullable for backward compat with existing data)
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- Create space shares table
CREATE TABLE IF NOT EXISTS space_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_id UUID NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission VARCHAR(20) NOT NULL CHECK (permission IN ('readonly', 'write', 'appendonly')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(space_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_space_shares_space_id ON space_shares(space_id);
CREATE INDEX IF NOT EXISTS idx_space_shares_user_id ON space_shares(user_id);
CREATE INDEX IF NOT EXISTS idx_spaces_owner_id ON spaces(owner_id);
