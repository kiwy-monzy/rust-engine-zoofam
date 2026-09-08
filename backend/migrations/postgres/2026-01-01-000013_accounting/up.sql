-- ================================================================
-- Accounting (invoices - references CRM and ERP)
-- ================================================================

CREATE TABLE accounting_invoices (
    id              TEXT PRIMARY KEY,
    invoice_number  TEXT NOT NULL UNIQUE,
    sales_order_id  TEXT REFERENCES erp_sales_orders(id),
    quote_id        TEXT REFERENCES crm_quotes(id),
    total           REAL NOT NULL,
    status          TEXT NOT NULL DEFAULT 'DRAFT',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
