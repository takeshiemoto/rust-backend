-- Add migration script here
CREATE
EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE email_verification_tokens
(
    id         SERIAL PRIMARY KEY,
    user_id    INTEGER   NOT NULL REFERENCES users (id),
    token      UUID      NOT NULL DEFAULT uuid_generate_v4(),
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);