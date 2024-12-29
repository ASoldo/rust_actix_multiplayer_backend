-- Add up migration script here
CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    sender UUID REFERENCES users(id) ON DELETE CASCADE,
    recipient UUID REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    activity_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

