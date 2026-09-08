-- ================================================================
-- Auth: Users, Sessions, Password Resets, Profiles, Revoked Tokens
-- ================================================================

CREATE TABLE gateway_users (
    id              VARCHAR(36)  PRIMARY KEY,
    email           VARCHAR(320) NOT NULL,
    email_lower     VARCHAR(320) NOT NULL UNIQUE,
    password_hash   VARCHAR(255) NOT NULL,
    display_name    VARCHAR(120) NOT NULL DEFAULT '',
    avatar_url      VARCHAR(500) NOT NULL DEFAULT '',
    is_active       BOOLEAN      NOT NULL DEFAULT TRUE,
    token_version   INTEGER      NOT NULL DEFAULT 0,
    first_name      VARCHAR(120) NOT NULL DEFAULT '',
    middle_name     VARCHAR(120) NOT NULL DEFAULT '',
    last_name       VARCHAR(120) NOT NULL DEFAULT '',
    username        VARCHAR(60)  NOT NULL DEFAULT '',
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX gateway_users_username_key ON gateway_users (username) WHERE username != '';

CREATE TABLE gateway_sessions (
    id                  VARCHAR(36)  PRIMARY KEY,
    user_id             VARCHAR(36)  NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    refresh_token_hash  VARCHAR(64)  NOT NULL,
    device              VARCHAR(200),
    user_agent          VARCHAR(500),
    ip                  VARCHAR(64),
    created_at          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    last_seen_at        TIMESTAMPTZ   NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ   NOT NULL,
    revoked_at          TIMESTAMPTZ
);
CREATE INDEX gateway_sessions_user_id_idx ON gateway_sessions (user_id);
CREATE INDEX gateway_sessions_refresh_token_hash_idx ON gateway_sessions (refresh_token_hash);

CREATE TABLE gateway_password_resets (
    id          VARCHAR(36)  PRIMARY KEY,
    user_id     VARCHAR(36)  NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    token_hash  VARCHAR(64)  NOT NULL,
    expires_at  TIMESTAMPTZ   NOT NULL,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ   NOT NULL DEFAULT now()
);
CREATE INDEX gateway_password_resets_token_hash_idx ON gateway_password_resets (token_hash);

CREATE TABLE gateway_profiles (
    user_id         VARCHAR(36)  PRIMARY KEY REFERENCES gateway_users(id) ON DELETE CASCADE,
    display_name    VARCHAR(120),
    phone           VARCHAR(32),
    location        VARCHAR(120),
    bio             TEXT,
    avatar_file_id  INTEGER,
    created_at      TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ   NOT NULL DEFAULT now()
);

CREATE TABLE gateway_revoked_tokens (
    jti         VARCHAR(36)  PRIMARY KEY,
    user_id     VARCHAR(36)  NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ    NOT NULL,
    revoked_at  TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE INDEX gateway_revoked_tokens_user_id_idx ON gateway_revoked_tokens (user_id);
CREATE INDEX gateway_revoked_tokens_expires_at_idx ON gateway_revoked_tokens (expires_at);
