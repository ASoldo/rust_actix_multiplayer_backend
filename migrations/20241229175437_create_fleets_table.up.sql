-- Add up migration script here
CREATE TABLE IF NOT EXISTS fleets (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ships INT DEFAULT 0 CHECK (ships >= 0),
    fighters INT DEFAULT 0 CHECK (fighters >= 0),
    bombers INT DEFAULT 0 CHECK (bombers >= 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);
