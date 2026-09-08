-- ================================================================
-- System settings (key-value schema)
-- ================================================================

CREATE TABLE gateway_system (
    key         VARCHAR(128) PRIMARY KEY,
    value       TEXT         NOT NULL,
    updated_at  TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_by  VARCHAR(36)  REFERENCES gateway_users(id) ON DELETE SET NULL
);

INSERT INTO gateway_system (key, value) VALUES
    ('app_name', 'MkulimaLink'),
    ('maintenance_mode', 'false'),
    ('registration_enabled', 'true'),
    ('version', '0.1.0');
