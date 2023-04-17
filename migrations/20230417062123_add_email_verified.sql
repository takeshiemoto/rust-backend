-- Add migration script here
ALTER TABLE users ADD COLUMN email_verified boolean NOT NULL DEFAULT false;
