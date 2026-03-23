-- Add password_hash column for custom authentication
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);

-- Make keycloak_id nullable and role optional (not needed for custom auth)
ALTER TABLE users ALTER COLUMN keycloak_id DROP NOT NULL;
ALTER TABLE users ALTER COLUMN role DROP NOT NULL;
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'user';

-- Add default timestamps if missing
ALTER TABLE users ALTER COLUMN created_at SET DEFAULT NOW();
ALTER TABLE users ALTER COLUMN updated_at SET DEFAULT NOW();

-- Drop keycloak index since we're not using it anymore
DROP INDEX IF EXISTS idx_users_keycloak_id;
