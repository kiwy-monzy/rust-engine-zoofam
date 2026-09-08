-- ================================================================
-- Storage - user files
-- ================================================================

CREATE TABLE user_files (
    id              SERIAL PRIMARY KEY,
    user_id         VARCHAR(36)  NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    filename        VARCHAR(255) NOT NULL,
    original_name   VARCHAR(255) NOT NULL,
    mime_type       VARCHAR(100) NOT NULL DEFAULT 'application/octet-stream',
    size_bytes      INTEGER      NOT NULL DEFAULT 0,
    storage_path    VARCHAR(500) NOT NULL,
    collection      VARCHAR(64)  NOT NULL DEFAULT 'default',
    is_public       BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE INDEX user_files_user_id_idx ON user_files (user_id);
CREATE INDEX user_files_created_at_idx ON user_files (created_at DESC);
CREATE INDEX user_files_collection_idx ON user_files (collection);
