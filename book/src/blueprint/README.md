# Blueprint — Two Modules, Full Flow

This section describes the two business modules that live alongside the gateway's
auth, fleet, and maps subsystems: **Commerce / ERP** and **CRM**. Together they
cover the full lead-to-cash + procure-to-pay lifecycle with money stored in
cents, multi-tenancy, multi-currency, tax engine, and reporting.

## Documents

- **[ERP Flow](./erp-flow.md)** — master data, procurement, receiving,
  inventory ledger, batch/serial tracking, stock adjustments & transfers,
  sales orders, shipments/packages/returns, payments & reconciliation,
  assets, expenses, custom fields, and reporting (SQL queries for stock on
  hand, sales by period, receivables aging, inventory valuation, tax
  liability, top customers, profit & loss).
- **[CRM Flow](./crm-flow.md)** — lead → opportunity → customer 360
  (sub-contacts, addresses, activities, portal), quoting with money in
  cents and tax math, the quote → sales order → invoice conversion chain,
  payments, support tickets, and reporting (pipeline by stage, win rate by
  source, quote-to-close time, lifetime revenue, receivables aging, top
  performers, lead conversion funnel, activity velocity).

## How the two modules connect

```
Marketing       Sales            Warehouse           Finance
--------        --------         ----------          --------
  |               |                  |                  |
  v               v                  v                  v
crm_leads -> crm_opportunities   commerce_shipments  accounting_invoices
                |                                          ^
                v                                          |
           crm_customers -> crm_quotes -> commerce_sales_orders
                                                    |
                                                    v
                                         commerce_payments
                                         commerce_payment_applications
```

Both modules share:
- `org_id` (multi-tenant scoping)
- `currency_code` + `exchange_rate` on every monetary doc
- `*_minor INTEGER` money (no floats in accounting)
- The `converted_from_id` chain for full auditability

## Conventions (apply to both modules)

- **UUIDs as TEXT** (SQLite) / `UUID` (Postgres) — never expose internal sequence IDs.
- **Timestamps as TEXT ISO8601** (SQLite) / `TIMESTAMPTZ` (Postgres).
- **Money as `INTEGER` minor units (cents).** To display, divide by 100 and format.
- **Status enums as TEXT** with default values like `DRAFT`, `ACTIVE`, `OPEN`.
- **Append-only ledger** for inventory and (optionally) for payments.
- **`org_id` defaults to `org_default`** so single-tenant deployments just work.