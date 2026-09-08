-- ================================================================
-- ERP (Products, Procurement, Inventory, Sales)
-- ================================================================

CREATE TABLE erp_units (
    id      TEXT PRIMARY KEY,
    name    TEXT NOT NULL,
    symbol  TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_warehouses (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    location    TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_suppliers (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    contact_email   TEXT,
    contact_phone   TEXT,
    address         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_products (
    id              TEXT PRIMARY KEY,
    sku             TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    description     TEXT,
    unit_id         TEXT REFERENCES erp_units(id),
    product_type    TEXT NOT NULL DEFAULT 'goods',
    cost_price      REAL DEFAULT 0,
    selling_price   REAL DEFAULT 0,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX erp_products_sku_idx ON erp_products (sku);
CREATE INDEX erp_products_type_idx ON erp_products (product_type);

CREATE TABLE erp_procurement_requests (
    id              TEXT PRIMARY KEY,
    request_number  TEXT NOT NULL UNIQUE,
    supplier_id     TEXT REFERENCES erp_suppliers(id),
    status          TEXT NOT NULL DEFAULT 'DRAFT',
    total           REAL NOT NULL DEFAULT 0,
    notes           TEXT,
    requested_by    TEXT NOT NULL REFERENCES gateway_users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_procurement_lines (
    id              TEXT PRIMARY KEY,
    request_id      TEXT NOT NULL REFERENCES erp_procurement_requests(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES erp_products(id),
    quantity        REAL NOT NULL,
    unit_price      REAL NOT NULL,
    notes           TEXT
);

CREATE TABLE erp_purchase_orders (
    id              TEXT PRIMARY KEY,
    po_number       TEXT NOT NULL UNIQUE,
    supplier_id     TEXT NOT NULL REFERENCES erp_suppliers(id),
    status          TEXT NOT NULL DEFAULT 'DRAFT',
    total           REAL NOT NULL DEFAULT 0,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_purchase_order_lines (
    id              TEXT PRIMARY KEY,
    po_id           TEXT NOT NULL REFERENCES erp_purchase_orders(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES erp_products(id),
    quantity        REAL NOT NULL,
    unit_price      REAL NOT NULL,
    notes           TEXT
);

CREATE TABLE erp_goods_receipts (
    id              TEXT PRIMARY KEY,
    receipt_number  TEXT NOT NULL UNIQUE,
    po_id           TEXT REFERENCES erp_purchase_orders(id),
    warehouse_id    TEXT NOT NULL REFERENCES erp_warehouses(id),
    received_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_goods_receipt_lines (
    id              TEXT PRIMARY KEY,
    receipt_id      TEXT NOT NULL REFERENCES erp_goods_receipts(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES erp_products(id),
    quantity        REAL NOT NULL,
    notes           TEXT
);

CREATE TABLE erp_inventory_transactions (
    id              TEXT PRIMARY KEY,
    product_id      TEXT NOT NULL REFERENCES erp_products(id),
    warehouse_id    TEXT NOT NULL REFERENCES erp_warehouses(id),
    transaction_type TEXT NOT NULL,
    quantity        REAL NOT NULL,
    reference_type  TEXT,
    reference_id    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX inv_tx_pw_idx ON erp_inventory_transactions(product_id, warehouse_id);
CREATE INDEX inv_tx_created_idx ON erp_inventory_transactions(created_at DESC);
CREATE INDEX inv_tx_ref_idx ON erp_inventory_transactions(reference_type, reference_id);

CREATE TABLE erp_sales_orders (
    id              TEXT PRIMARY KEY,
    so_number       TEXT NOT NULL UNIQUE,
    customer_name   TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'DRAFT',
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_sales_lines (
    id              TEXT PRIMARY KEY,
    sales_order_id  TEXT NOT NULL REFERENCES erp_sales_orders(id) ON DELETE CASCADE,
    product_id      TEXT NOT NULL REFERENCES erp_products(id),
    quantity        REAL NOT NULL,
    unit_price      REAL NOT NULL,
    notes           TEXT
);

CREATE TABLE erp_shipments (
    id              TEXT PRIMARY KEY,
    sales_order_id  TEXT NOT NULL REFERENCES erp_sales_orders(id) ON DELETE CASCADE,
    warehouse_id    TEXT REFERENCES erp_warehouses(id),
    shipped_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    notes           TEXT
);

CREATE TABLE erp_assets (
    id              TEXT PRIMARY KEY,
    asset_number    TEXT NOT NULL UNIQUE,
    product_id      TEXT REFERENCES erp_products(id),
    name            TEXT NOT NULL,
    acquisition_cost REAL NOT NULL,
    acquisition_date TIMESTAMPTZ NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active',
    location_id     TEXT REFERENCES erp_warehouses(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE erp_expenses (
    id              TEXT PRIMARY KEY,
    expense_number  TEXT NOT NULL UNIQUE,
    category        TEXT NOT NULL,
    description     TEXT NOT NULL,
    amount          REAL NOT NULL,
    expense_date    TIMESTAMPTZ NOT NULL,
    status          TEXT NOT NULL DEFAULT 'DRAFT',
    approved_by     TEXT,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed reference data
INSERT INTO erp_units (id, name, symbol) VALUES
    ('00000000-0000-0000-0000-000000000001','Piece','pcs'),
    ('00000000-0000-0000-0000-000000000002','Kilogram','kg'),
    ('00000000-0000-0000-0000-000000000003','Liter','l');

INSERT INTO erp_warehouses (id, name, location) VALUES
    ('00000000-0000-0000-0000-000000000011','Main Warehouse','Dar es Salaam'),
    ('00000000-0000-0000-0000-000000000012','Cold Store','Dar es Salaam - Port');

INSERT INTO erp_suppliers (id, name, contact_email) VALUES
    ('00000000-0000-0000-0000-000000000021','Acme Supplies','acme@example.tz'),
    ('00000000-0000-0000-0000-000000000022','Global Trade Ltd','trade@example.tz');

INSERT INTO erp_products (id, sku, name, description, unit_id, product_type, is_active) VALUES
    ('00000000-0000-0000-0000-000000000031','SKU-LAPTOP-001','Laptop Pro 14','14 inch business laptop','00000000-0000-0000-0000-000000000001','goods',1),
    ('00000000-0000-0000-0000-000000000032','SKU-CEMENT-001','Cement 50kg','Portland cement bag','00000000-0000-0000-0000-000000000002','goods',1),
    ('00000000-0000-0000-0000-000000000033','SKU-SERVICE-001','Consulting Hour','Professional service','00000000-0000-0000-0000-000000000001','service',1);
