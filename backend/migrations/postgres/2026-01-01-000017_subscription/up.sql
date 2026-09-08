-- ================================================================
-- Subscription & Plans module
-- ================================================================

CREATE TABLE subscription_plans (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    slug            TEXT NOT NULL UNIQUE,
    description     TEXT,
    price_monthly   REAL NOT NULL DEFAULT 0,
    price_yearly    REAL NOT NULL DEFAULT 0,
    currency        TEXT NOT NULL DEFAULT 'TZS',
    max_users       INTEGER NOT NULL DEFAULT 1,
    max_products    INTEGER NOT NULL DEFAULT 100,
    max_orders      INTEGER NOT NULL DEFAULT 100,
    features        TEXT NOT NULL DEFAULT '[]',
    is_active       INTEGER NOT NULL DEFAULT 1,
    is_public       INTEGER NOT NULL DEFAULT 1,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX subscription_plans_active_idx ON subscription_plans (is_active, is_public);
CREATE INDEX subscription_plans_sort_idx ON subscription_plans (sort_order);

CREATE TABLE organization_subscriptions (
    id                  TEXT PRIMARY KEY,
    org_id              TEXT NOT NULL REFERENCES marketplace_organizations(id) ON DELETE CASCADE,
    plan_id             TEXT NOT NULL REFERENCES subscription_plans(id),
    status              TEXT NOT NULL DEFAULT 'trial',
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end   TIMESTAMPTZ NOT NULL,
    cancel_at_period_end INTEGER NOT NULL DEFAULT 0,
    payment_method      TEXT,
    trial_ends_at       TIMESTAMPTZ,
    cancelled_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(org_id, status)
);
CREATE INDEX org_subs_org_idx ON organization_subscriptions (org_id);
CREATE INDEX org_subs_status_idx ON organization_subscriptions (status);
CREATE INDEX org_subs_period_end_idx ON organization_subscriptions (current_period_end);

CREATE TABLE subscription_invoices (
    id              TEXT PRIMARY KEY,
    subscription_id TEXT NOT NULL REFERENCES organization_subscriptions(id) ON DELETE CASCADE,
    org_id          TEXT NOT NULL REFERENCES marketplace_organizations(id) ON DELETE CASCADE,
    amount          REAL NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'TZS',
    status          TEXT NOT NULL DEFAULT 'draft',
    due_date        TIMESTAMPTZ NOT NULL,
    paid_at         TIMESTAMPTZ,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX sub_inv_sub_idx ON subscription_invoices (subscription_id);
CREATE INDEX sub_inv_org_idx ON subscription_invoices (org_id);
CREATE INDEX sub_inv_status_idx ON subscription_invoices (status);

-- Default plans
INSERT INTO subscription_plans (id, name, slug, description, price_monthly, price_yearly, max_users, max_products, max_orders, features, sort_order) VALUES
    ('00000000-0000-0000-0000-000000000101', 'Free', 'free', 'For small farmers getting started', 0, 0, 1, 10, 10, '["basic_listing","community_support"]', 1),
    ('00000000-0000-0000-0000-000000000102', 'Starter', 'starter', 'For growing businesses', 29000, 290000, 3, 100, 100, '["basic_listing","priority_support","analytics"]', 2),
    ('00000000-0000-0000-0000-000000000103', 'Professional', 'professional', 'For established organizations', 99000, 990000, 10, 1000, 1000, '["basic_listing","priority_support","analytics","api_access","custom_branding"]', 3),
    ('00000000-0000-0000-0000-000000000104', 'Enterprise', 'enterprise', 'For large operations', 299000, 2990000, 100, 10000, 10000, '["basic_listing","priority_support","analytics","api_access","custom_branding","dedicated_manager","sla"]', 4);
