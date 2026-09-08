-- ================================================================
-- Support tickets
-- ================================================================

CREATE TABLE support_tickets (
    id          VARCHAR(36)  PRIMARY KEY,
    subject     VARCHAR(255) NOT NULL,
    description TEXT         NOT NULL DEFAULT '',
    status      VARCHAR(20)  NOT NULL DEFAULT 'open',
    priority    VARCHAR(20)  NOT NULL DEFAULT 'normal',
    category    VARCHAR(64)  NOT NULL DEFAULT 'general',
    created_by  VARCHAR(36)  NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    assigned_to VARCHAR(36)  REFERENCES gateway_users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ    NOT NULL DEFAULT now(),
    closed_at   TIMESTAMPTZ
);
CREATE INDEX support_tickets_created_by_idx ON support_tickets (created_by);
CREATE INDEX support_tickets_status_idx ON support_tickets (status);
CREATE INDEX support_tickets_assigned_to_idx ON support_tickets (assigned_to);
