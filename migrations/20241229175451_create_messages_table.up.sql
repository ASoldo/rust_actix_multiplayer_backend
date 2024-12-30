-- Add up migration script here
CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    sender UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL CHECK (length(content) > 0),
    activity_type VARCHAR(50) NOT NULL CHECK (length(activity_type) > 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);
