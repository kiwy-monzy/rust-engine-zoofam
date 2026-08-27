-- Commerce ERP — products, procurement, purchasing, goods receipt, inventory ledger, sales, assets, expenses.
-- This migration is intentionally aligned with `models/src/schema.rs` so the existing
-- Diesel table macros and the existing models/controller/routes compile unchanged.
-- Improvements (org_id, money in minor, currency_code, tax_id, etc.) are added as
-- OPTIONAL additive columns so existing code keeps working.

CREATE TABLE IF NOT EXISTS commerce_units (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS commerce_warehouses (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS commerce_suppliers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    contact_email TEXT,
    contact_phone TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS commerce_products (
    id TEXT PRIMARY KEY,
    sku TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    unit_id TEXT REFERENCES commerce_units(id),
    product_type TEXT NOT NULL DEFAULT 'goods',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX IF NOT EXISTS commerce_products_sku_idx ON commerce_products(sku);
CREATE INDEX IF NOT EXISTS commerce_products_type_idx ON commerce_products(product_type);

CREATE TABLE IF NOT EXISTS commerce_procurement_requests (
    id TEXT PRIMARY KEY,
    request_number TEXT NOT NULL UNIQUE,
    requested_by TEXT,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS commerce_procurement_lines (
    id TEXT PRIMARY KEY,
    procurement_id TEXT NOT NULL REFERENCES commerce_procurement_requests(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES commerce_products(id),
    quantity REAL NOT NULL DEFAULT 1,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS commerce_purchase_orders (
    id TEXT PRIMARY KEY,
    po_number TEXT NOT NULL UNIQUE,
    supplier_id TEXT REFERENCES commerce_suppliers(id),
    status TEXT NOT NULL DEFAULT 'DRAFT',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS commerce_purchase_order_lines (
    id TEXT PRIMARY KEY,
    purchase_order_id TEXT NOT NULL REFERENCES commerce_purchase_orders(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES commerce_products(id),
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS commerce_goods_receipts (
    id TEXT PRIMARY KEY,
    receipt_number TEXT NOT NULL UNIQUE,
    purchase_order_id TEXT REFERENCES commerce_purchase_orders(id),
    warehouse_id TEXT REFERENCES commerce_warehouses(id),
    received_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS commerce_goods_receipt_lines (
    id TEXT PRIMARY KEY,
    receipt_id TEXT NOT NULL REFERENCES commerce_goods_receipts(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES commerce_products(id),
    quantity REAL NOT NULL,
    notes TEXT
);

CREATE TABLE IF NOT EXISTS commerce_inventory_transactions (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES commerce_products(id),
    warehouse_id TEXT NOT NULL REFERENCES commerce_warehouses(id),
    transaction_type TEXT NOT NULL,
    quantity REAL NOT NULL,
    reference_type TEXT,
    reference_id TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX IF NOT EXISTS inv_tx_pw_idx ON commerce_inventory_transactions(product_id, warehouse_id);
CREATE INDEX IF NOT EXISTS inv_tx_created_idx ON commerce_inventory_transactions(created_at DESC);
CREATE INDEX IF NOT EXISTS inv_tx_ref_idx ON commerce_inventory_transactions(reference_type, reference_id);

CREATE TABLE IF NOT EXISTS commerce_sales_orders (
    id TEXT PRIMARY KEY,
    so_number TEXT NOT NULL UNIQUE,
    customer_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE TABLE IF NOT EXISTS commerce_sales_lines (
    id TEXT PRIMARY KEY,
    sales_order_id TEXT NOT NULL REFERENCES commerce_sales_orders(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES commerce_products(id),
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    notes TEXT
);
CREATE TABLE IF NOT EXISTS commerce_shipments (
    id TEXT PRIMARY KEY,
    sales_order_id TEXT NOT NULL REFERENCES commerce_sales_orders(id) ON DELETE CASCADE,
    warehouse_id TEXT REFERENCES commerce_warehouses(id),
    shipped_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    notes TEXT
);

CREATE TABLE IF NOT EXISTS commerce_assets (
    id TEXT PRIMARY KEY,
    asset_number TEXT NOT NULL UNIQUE,
    product_id TEXT REFERENCES commerce_products(id),
    name TEXT NOT NULL,
    acquisition_cost REAL NOT NULL,
    acquisition_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    location_id TEXT REFERENCES commerce_warehouses(id),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

CREATE TABLE IF NOT EXISTS commerce_expenses (
    id TEXT PRIMARY KEY,
    expense_number TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    description TEXT NOT NULL,
    amount REAL NOT NULL,
    expense_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    approved_by TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- Permissions for commerce
INSERT OR IGNORE INTO gateway_permissions (module, action, description) VALUES
    ('commerce', 'read',  'View commerce modules'),
    ('commerce', 'write', 'Manage commerce modules'),
    ('commerce', 'admin', 'Admin commerce'),
    ('products', 'read',   'View products'),
    ('procurement', 'read','View procurement'),
    ('purchasing', 'read', 'View purchasing'),
    ('inventory',  'read', 'View inventory'),
    ('sales',      'read', 'View sales'),
    ('asset',      'read', 'View assets'),
    ('expense',    'read', 'View expenses');
INSERT OR IGNORE INTO gateway_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM gateway_roles r CROSS JOIN gateway_permissions p
 WHERE r.name='admin' AND p.module IN ('commerce','products','procurement','purchasing','inventory','sales','asset','expense');
INSERT OR IGNORE INTO gateway_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM gateway_roles r JOIN gateway_permissions p ON p.module='commerce' AND p.action='read'
 WHERE r.name='employee';

-- Seed reference data
INSERT OR IGNORE INTO commerce_units (id, name, symbol) VALUES
    ('00000000-0000-0000-0000-000000000001','Piece','pcs'),
    ('00000000-0000-0000-0000-000000000002','Kilogram','kg'),
    ('00000000-0000-0000-0000-000000000003','Liter','l');
INSERT OR IGNORE INTO commerce_warehouses (id, name, location) VALUES
    ('00000000-0000-0000-0000-000000000011','Main Warehouse','Dar es Salaam'),
    ('00000000-0000-0000-0000-000000000012','Cold Store','Dar es Salaam - Port');
INSERT OR IGNORE INTO commerce_suppliers (id, name, contact_email) VALUES
    ('00000000-0000-0000-0000-000000000021','Acme Supplies','acme@example.tz'),
    ('00000000-0000-0000-0000-000000000022','Global Trade Ltd','trade@example.tz');
INSERT OR IGNORE INTO commerce_products (id, sku, name, description, unit_id, product_type, is_active) VALUES
    ('00000000-0000-0000-0000-000000000031','SKU-LAPTOP-001','Laptop Pro 14','14 inch business laptop','00000000-0000-0000-0000-000000000001','goods',1),
    ('00000000-0000-0000-0000-000000000032','SKU-CEMENT-001','Cement 50kg','Portland cement bag','00000000-0000-0000-0000-000000000002','goods',1),
    ('00000000-0000-0000-0000-000000000033','SKU-SERVICE-001','Consulting Hour','Professional service','00000000-0000-0000-0000-000000000001','service',1);