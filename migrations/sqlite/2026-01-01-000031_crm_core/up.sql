-- CRM — Lead -> Opportunity -> Customer -> Quote -> Sales (feeds ERP Sales)
-- Aligned with the existing `models/src/schema.rs` so the existing code compiles unchanged.

CREATE TABLE IF NOT EXISTS crm_leads (
    id TEXT PRIMARY KEY,
    lead_number TEXT NOT NULL UNIQUE,
    company_name TEXT,
    contact_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    source TEXT,
    status TEXT NOT NULL DEFAULT 'NEW',
    assigned_to TEXT,
    product_id TEXT REFERENCES commerce_products(id),
    estimated_value REAL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS crm_opportunities (
    id TEXT PRIMARY KEY,
    opp_number TEXT NOT NULL UNIQUE,
    lead_id TEXT REFERENCES crm_leads(id),
    customer_id TEXT,
    stage TEXT NOT NULL DEFAULT 'PROSPECT',
    probability REAL DEFAULT 0.3,
    amount REAL,
    expected_close TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS crm_customers (
    id TEXT PRIMARY KEY,
    customer_number TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    address TEXT,
    lead_id TEXT REFERENCES crm_leads(id),
    opportunity_id TEXT REFERENCES crm_opportunities(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS crm_contacts (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES crm_customers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    email TEXT,
    role TEXT,
    phone TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS crm_activities (
    id TEXT PRIMARY KEY,
    related_type TEXT NOT NULL,
    related_id TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    subject TEXT NOT NULL,
    due_at TEXT,
    done INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS crm_quotes (
    id TEXT PRIMARY KEY,
    quote_number TEXT NOT NULL UNIQUE,
    customer_id TEXT REFERENCES crm_customers(id),
    opportunity_id TEXT REFERENCES crm_opportunities(id),
    status TEXT NOT NULL DEFAULT 'DRAFT',
    valid_until TEXT,
    total REAL DEFAULT 0,
    sales_order_id TEXT REFERENCES commerce_sales_orders(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS crm_quote_lines (
    id TEXT PRIMARY KEY,
    quote_id TEXT NOT NULL REFERENCES crm_quotes(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES commerce_products(id),
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS accounting_invoices (
    id TEXT PRIMARY KEY,
    invoice_number TEXT NOT NULL UNIQUE,
    sales_order_id TEXT REFERENCES commerce_sales_orders(id),
    quote_id TEXT REFERENCES crm_quotes(id),
    total REAL NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Permissions for CRM
INSERT OR IGNORE INTO gateway_permissions (module, action, description) VALUES
    ('crm','read','View CRM'),('crm','write','Manage CRM'),('crm','admin','Admin CRM');
INSERT OR IGNORE INTO gateway_role_permissions (role_id, permission_id)
SELECT r.id,p.id FROM gateway_roles r CROSS JOIN gateway_permissions p WHERE r.name='admin' AND p.module='crm';
INSERT OR IGNORE INTO gateway_role_permissions (role_id, permission_id)
SELECT r.id,p.id FROM gateway_roles r JOIN gateway_permissions p ON p.module='crm' AND p.action='read' WHERE r.name='employee';