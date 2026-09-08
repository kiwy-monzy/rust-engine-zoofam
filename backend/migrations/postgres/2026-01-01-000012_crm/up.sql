-- ================================================================
-- CRM (Leads, Opportunities, Customers)
-- ================================================================

CREATE TABLE crm_leads (
    id              TEXT PRIMARY KEY,
    lead_number     TEXT NOT NULL UNIQUE,
    company_name    TEXT,
    contact_name    TEXT NOT NULL,
    email           TEXT,
    phone           TEXT,
    source          TEXT,
    status          TEXT NOT NULL DEFAULT 'NEW',
    assigned_to     TEXT,
    product_id      TEXT REFERENCES erp_products(id),
    estimated_value REAL,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE crm_opportunities (
    id              TEXT PRIMARY KEY,
    opp_number      TEXT NOT NULL UNIQUE,
    lead_id         TEXT REFERENCES crm_leads(id),
    customer_id     TEXT,
    stage           TEXT NOT NULL DEFAULT 'PROSPECT',
    probability     REAL DEFAULT 0.3,
    amount          REAL,
    expected_close  TIMESTAMPTZ,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE crm_customers (
    id              TEXT PRIMARY KEY,
    customer_number TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    email           TEXT,
    phone           TEXT,
    address         TEXT,
    lead_id         TEXT REFERENCES crm_leads(id),
    opportunity_id  TEXT REFERENCES crm_opportunities(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE crm_contacts (
    id              TEXT PRIMARY KEY,
    customer_id     TEXT NOT NULL REFERENCES crm_customers(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    email           TEXT,
    role            TEXT,
    phone           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE crm_activities (
    id              TEXT PRIMARY KEY,
    related_type    TEXT NOT NULL,
    related_id      TEXT NOT NULL,
    activity_type   TEXT NOT NULL,
    subject         TEXT NOT NULL,
    due_at          TIMESTAMPTZ,
    done            INTEGER NOT NULL DEFAULT 0,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE crm_quotes (
    id              TEXT PRIMARY KEY,
    quote_number    TEXT NOT NULL UNIQUE,
    customer_id     TEXT REFERENCES crm_customers(id),
    opportunity_id  TEXT REFERENCES crm_opportunities(id),
    status          TEXT NOT NULL DEFAULT 'DRAFT',
    valid_until     TIMESTAMPTZ,
    total           REAL DEFAULT 0,
    sales_order_id  TEXT REFERENCES erp_sales_orders(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE crm_quote_lines (
    id              TEXT PRIMARY KEY,
    quote_id        TEXT NOT NULL REFERENCES crm_quotes(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES erp_products(id),
    quantity        REAL NOT NULL,
    unit_price      REAL NOT NULL,
    notes           TEXT
);
