-- ================================================================
-- Wallet passes (Apple Wallet)
-- ================================================================

CREATE TABLE dmc_wallet_passes (
    id              VARCHAR(36)  PRIMARY KEY,
    pass_type       VARCHAR(100) NOT NULL,
    serial_number   VARCHAR(100) NOT NULL UNIQUE,
    auth_token      VARCHAR(255) NOT NULL,
    user_id         VARCHAR(36)  REFERENCES gateway_users(id) ON DELETE SET NULL,
    data            TEXT         NOT NULL DEFAULT '{}',
    is_active       BOOLEAN      NOT NULL DEFAULT TRUE,
    last_updated    TIMESTAMPTZ    NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE INDEX dmc_wallet_passes_user_id_idx ON dmc_wallet_passes (user_id);
CREATE INDEX dmc_wallet_passes_serial_idx ON dmc_wallet_passes (serial_number);

CREATE TABLE dmc_wallet_registrations (
    id              VARCHAR(36)  PRIMARY KEY,
    device_id       VARCHAR(255) NOT NULL,
    pass_type       VARCHAR(100) NOT NULL,
    serial_number   VARCHAR(100) NOT NULL,
    push_token      TEXT,
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT now(),
    UNIQUE (device_id, pass_type, serial_number)
);
CREATE INDEX dmc_wallet_registrations_pass_idx ON dmc_wallet_registrations (pass_type, serial_number);
