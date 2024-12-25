-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  email TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,       -- hashed password
  created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);
