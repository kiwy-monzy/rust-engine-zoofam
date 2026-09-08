-- ================================================================
-- Vault
-- ================================================================

CREATE TABLE gateway_vault (
    id              VARCHAR(36)  PRIMARY KEY,
    user_id         VARCHAR(36)  NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    service         VARCHAR(64)  NOT NULL,
    kind            VARCHAR(64)  NOT NULL DEFAULT 'generic',
    name            VARCHAR(255) NOT NULL,
    secret          TEXT         NOT NULL,
    meta            TEXT         NOT NULL,
    is_active       BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ    NOT NULL DEFAULT now()
);
CREATE INDEX gateway_vault_user_id_idx ON gateway_vault (user_id);
CREATE INDEX gateway_vault_service_idx ON gateway_vault (service);
CREATE INDEX gateway_vault_kind_idx ON gateway_vault (kind);
CREATE INDEX gateway_vault_is_active_idx ON gateway_vault (is_active);
