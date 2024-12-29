-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,                 -- Unique username
  email TEXT NOT NULL UNIQUE,                    -- Unique email
  password TEXT NOT NULL,                        -- Hashed password
  public_key_pem TEXT,                           -- Public key for ActivityPub
  created_at TIMESTAMP WITH TIME ZONE DEFAULT now() -- Timestamp of user creation
);

-- Create indexes for optimized lookups (optional)
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
