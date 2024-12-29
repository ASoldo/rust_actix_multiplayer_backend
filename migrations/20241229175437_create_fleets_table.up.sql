-- Add up migration script here
CREATE TABLE IF NOT EXISTS fleets (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    ships INT DEFAULT 0,
    fighters INT DEFAULT 0,
    bombers INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

