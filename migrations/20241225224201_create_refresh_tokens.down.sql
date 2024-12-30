-- Add down migration script here
ALTER TABLE refresh_tokens DROP CONSTRAINT fk_user;
DROP TABLE IF EXISTS refresh_tokens;
