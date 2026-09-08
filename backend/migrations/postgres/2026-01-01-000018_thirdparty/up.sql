-- ================================================================
-- Thirdparty integrations module
-- ================================================================

-- Thirdparty integration logs (for audit trail)
CREATE TABLE thirdparty_logs (
    id              TEXT PRIMARY KEY,
    provider        TEXT NOT NULL,
    action          TEXT NOT NULL,
    status          TEXT NOT NULL,
    request_data    TEXT,
    response_data   TEXT,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX thirdparty_logs_provider_idx ON thirdparty_logs (provider);
CREATE INDEX thirdparty_logs_action_idx ON thirdparty_logs (action);
CREATE INDEX thirdparty_logs_created_idx ON thirdparty_logs (created_at DESC);

-- Thirdparty provider configuration (runtime enable/disable)
CREATE TABLE thirdparty_config (
    provider        TEXT PRIMARY KEY,
    is_enabled      INTEGER NOT NULL DEFAULT 1,
    config_json     TEXT NOT NULL DEFAULT '{}',
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by      TEXT REFERENCES gateway_users(id) ON DELETE SET NULL
);

-- Insert default thirdparty providers
INSERT INTO thirdparty_config (provider, is_enabled, config_json) VALUES
    ('clickpesa', 1, '{"features":["payments","payouts","billpay","exchange","links"]}'),
    ('beem', 1, '{"features":["sms","ussd"]}'),
    ('notifty', 0, '{"features":["sms","ussd"]}');
