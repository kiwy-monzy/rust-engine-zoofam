-- ================================================================
-- Releases
-- ================================================================

CREATE TABLE gateway_releases (
    id          SERIAL PRIMARY KEY,
    version     VARCHAR(50)  NOT NULL UNIQUE,
    platform    VARCHAR(32)  NOT NULL DEFAULT 'linux-x86_64',
    filename    VARCHAR(255) NOT NULL,
    file_size   INTEGER,
    sha256      VARCHAR(64),
    changelog   TEXT,
    download_count INTEGER NOT NULL DEFAULT 0,
    created_by  VARCHAR(36)  REFERENCES gateway_users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE INDEX gateway_releases_version_idx ON gateway_releases (version);
