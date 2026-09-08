# Commerce / ERP Module — Full Flow

The Commerce module covers the full **source-to-report** lifecycle: master data setup,
purchasing, receiving, inventory movement, sales, fulfillment, returns, payments, expenses,
and reporting.

---

## 1. Master data setup

Before any transaction can happen, the org seeds its reference data.

| Entity | Purpose | Example |
|---|---|---|
| `gateway_organizations` | Multi-tenant root (one row per tenant) | `org_default` |
| `commerce_unit_groups` | Group of units (e.g. *Mass*: kg, g, lb) | `Mass` |
| `commerce_units` | Individual UoM with `factor` to base | `kg` (base, factor=1), `g` (factor=0.001) |
| `commerce_warehouses` | Physical locations that hold stock | `Main Warehouse` |
| `commerce_suppliers` | Vendors (purchase from) | `Acme Supplies` |
| `commerce_taxes` | Tax definitions (rate, type, default) | `VAT Standard 18%` |
| `commerce_expense_accounts` | Categories for expenses | `Travel`, `Office Supplies` |
| `commerce_price_lists` | Optional: per-market price books | `Retail TZS`, `Wholesale USD` |
| `commerce_products` | **Item master** — every SKU the org trades | `SKU-LAPTOP-001` |

### Product master (`commerce_products`) — key fields

| Field | Why it matters |
|---|---|
| `sku` (unique per `org_id`) | Stock-keeping unit, the human key |
| `product_type` | `goods` (track inventory) or `service` (no stock) |
| `tax_id` | Default tax applied to every line involving this item |
| `is_taxable` | Quick toggle, mirrors tax_id linkage |
| `rate_minor` | Default sell price in **cents** (INTEGER) |
| `purchase_rate_minor` | Default buy price in **cents** |
| `cost_minor` | Standard / moving-average cost |
| `track_inventory` | If false, item never writes to the inventory ledger |
| `track_batch` / `track_serial` | Forces per-batch / per-unit stock movement |
| `can_be_sold` / `can_be_purchased` | If false, hidden from sales/POS or purchase UI |
| `reorder_level` / `reorder_point` | Triggers low-stock alerts |
| `opening_stock` | Seeded into the ledger on item creation |
| `vendor_id` | Default supplier for reordering |

---

## 2. Procurement lifecycle

```
[User in Procurement UI]
        |
        v
commerce_procurement_requests  (DRAFT -> SUBMITTED -> APPROVED)
        | one-to-many
        v
commerce_procurement_lines     (product_id, qty)
        |
        v  (Approve action creates PO)
commerce_purchase_orders       (DRAFT -> SENT -> PARTIAL -> RECEIVED)
        | one-to-many
        v
commerce_purchase_order_lines  (product_id, qty, unit_price_minor, tax_*)
```

Each PO line carries the **money in minor units** for the buy side:
- `unit_price_minor` (cents)
- `discount_minor` (cents)
- `tax_id` + `tax_percentage` + `tax_amount_minor`
- `line_total_minor` = `qty * unit_price_minor - discount_minor + tax_amount_minor`

---

## 3. Receiving & inventory movement

```
[Goods arrive at warehouse]
        |
        v
commerce_goods_receipts  (links to PO + warehouse, sets received_at)
        | one-to-many
        v
commerce_goods_receipt_lines  (product_id, qty, optional batch_id)
        |
        v  (side effect: inventory ledger entry)
commerce_inventory_transactions
    product_id, warehouse_id, transaction_type = 'RECEIPT'
    quantity = +received, reference_type = 'GOODS_RECEIPT', reference_id
```

### Inventory ledger (`commerce_inventory_transactions`) — append-only

| `transaction_type` | Sign of `quantity` | When written |
|---|---|---|
| `RECEIPT` | + | Goods receipt posted |
| `SHIPMENT` | - | Sales shipment leaves the warehouse |
| `ADJUSTMENT` | +/- | Manual stock count correction |
| `TRANSFER_OUT` | - | Inter-warehouse transfer posts the leaving side |
| `TRANSFER_IN` | + | Inter-warehouse transfer posts the arriving side |
| `RETURN` | + | Customer return received back into stock |
| `ASSET` | - | Item capitalized to fixed asset |

**Stock-on-hand is a derived view, never stored:**
```sql
SELECT product_id, warehouse_id, SUM(quantity) AS on_hand
FROM commerce_inventory_transactions
GROUP BY product_id, warehouse_id;
```

### Batch & serial tracking

For items with `track_batch = 1`:
- `commerce_goods_receipt_lines.batch_id` is filled in
- `commerce_inventory_batches` stores the batch (number, manufactured_at, expires_at, qty, rate_minor)
- Per-batch stock = `SUM(received) - SUM(shipped) - SUM(adjusted)`

For items with `track_serial = 1`:
- `commerce_inventory_serials` stores each unit (status: `in_stock | reserved | sold | returned`)

### Stock adjustments & transfers

| Operation | Tables touched | Ledger effect |
|---|---|---|
| Manual count | `commerce_stock_adjustments` + lines | one `ADJUSTMENT` row per line |
| Inter-warehouse | `commerce_stock_transfers` + lines | one `TRANSFER_OUT` row + one `TRANSFER_IN` row |

---

## 4. Sales lifecycle

```
[Sales rep / CRM quote conversion]
        |
        v
commerce_sales_orders  (DRAFT -> CONFIRMED -> PARTIAL -> SHIPPED -> INVOICED -> PAID)
   customer_id (crm_customers), customer_name (denormalized)
   currency_code, exchange_rate
   sub_total_minor, tax_total_minor, discount_minor, shipping_minor, total_minor
        | one-to-many
        v
commerce_sales_lines  (product_id, qty, unit_price_minor, tax_*, line_total_minor)
        |
        v  (warehouse picker releases the order)
commerce_shipments  (shipment_number, carrier, tracking_number, shipping_address, shipping_charge_minor)
        | one-to-many
        v
commerce_packages     (package_number, tracking_number, total_quantity)
        |
        v  (side effect)
commerce_inventory_transactions (type = SHIPMENT, quantity = -shipped, ref = SALES_ORDER)
        |
        v  (carrier delivers)
status -> 'delivered'
```

### Returns

If the customer sends goods back:
```
commerce_sales_returns (return_number, sales_order_id, customer_id, reason, refund_status)
    | one-to-many
    v
commerce_sales_return_lines (product_id, qty, rate_minor)
    |
    v
commerce_inventory_transactions (type = RETURN, quantity = +returned, ref = SALES_RETURN)
```

---

## 5. Payments & reconciliation

```
[Customer pays invoice]
        |
        v
commerce_payments  (payment_number, customer_id, payment_mode, amount_minor, currency_code)
        | one-to-many
        v
commerce_payment_applications
    payment_id, invoice_id, sales_order_id, amount_applied_minor
```

A single payment can be **split across many invoices**. The `unused_minor` and
`refunded_minor` fields on the payment track the residual state. The invoice's
`payment_made_minor`, `credits_applied_minor`, and `balance_minor` (on
`accounting_invoices`) are updated by the application so reports show real
outstanding amounts.

---

## 6. Assets

When an item is purchased for internal use (not resale), it is **capitalized**:

```
commerce_assets  (asset_number, product_id, name, acquisition_cost_minor, acquisition_date, status, location_id)
    |
    v  (side effect: stock out)
commerce_inventory_transactions (type = ASSET, quantity = -1, ref = ASSET)
```

---

## 7. Expenses

```
[Employee submits an expense]
        |
        v
commerce_expenses  (expense_number, account_id, vendor_id, amount_minor, tax_*, total_minor,
                    currency_code, expense_date, status, is_billable, customer_id, invoice_id)
```

- `is_billable = 1` + `customer_id` set → expense will be re-billed on the next invoice for that customer
- `account_id` routes the expense into the right P&L category
- Approval workflow: `DRAFT -> SUBMITTED -> APPROVED -> REIMBURSED`

---

## 8. Custom fields

`custom_field_defs` and `custom_field_values` let admins add fields to any
entity (e.g. `crm_customers`, `commerce_products`) **without a code change**:

- Define: `entity='commerce_products', label='Color', data_type='select', options='["red","blue"]'`
- Set: `(customfield_id, entity_id=product.id, value='red')`
- `show_on_pdf = 1` → renderer pulls the value onto the printed invoice

---

## 9. Reporting & analytics

All reports are SQL views over the append-only ledger and the transactional tables.

### 9.1 Stock on hand

```sql
SELECT p.sku, p.name, w.name AS warehouse,
       COALESCE(SUM(t.quantity), 0) AS on_hand,
       COALESCE(SUM(t.quantity) * p.cost_minor / 100.0, 0) AS value
FROM commerce_products p
LEFT JOIN commerce_inventory_transactions t ON t.product_id = p.id
LEFT JOIN commerce_warehouses w ON w.id = t.warehouse_id
WHERE p.track_inventory = 1
GROUP BY p.id, w.id;
```

### 9.2 Sales by period (in major currency)

```sql
SELECT strftime('%Y-%m', created_at) AS month,
       currency_code,
       SUM(total_minor) / 100.0 AS revenue,
       SUM(tax_total_minor) / 100.0 AS tax_collected
FROM commerce_sales_orders
WHERE status IN ('CONFIRMED','SHIPPED','INVOICED','PAID')
GROUP BY month, currency_code
ORDER BY month DESC;
```

### 9.3 Receivables aging

```sql
SELECT c.id, c.name,
       SUM(i.balance_minor) / 100.0 AS outstanding,
       SUM(CASE WHEN i.due_date >= date('now') THEN i.balance_minor ELSE 0 END) / 100.0 AS current_due,
       SUM(CASE WHEN i.due_date <  date('now') AND julianday('now') - julianday(i.due_date) <= 30 THEN i.balance_minor ELSE 0 END) / 100.0 AS days_1_30,
       SUM(CASE WHEN julianday('now') - julianday(i.due_date) BETWEEN 31 AND 60  THEN i.balance_minor ELSE 0 END) / 100.0 AS days_31_60,
       SUM(CASE WHEN julianday('now') - julianday(i.due_date) >  60                THEN i.balance_minor ELSE 0 END) / 100.0 AS days_60_plus
FROM accounting_invoices i
JOIN crm_customers c ON c.id = i.customer_id
WHERE i.status NOT IN ('PAID','VOID')
GROUP BY c.id
ORDER BY outstanding DESC;
```

### 9.4 Inventory valuation (FIFO / standard cost)

- **Standard cost** uses `commerce_products.cost_minor`
- **FIFO cost** is `SUM(per-batch quantity * batch.rate_minor) / total_qty` for that product

### 9.5 Tax liability

```sql
SELECT strftime('%Y-%m', t.created_at) AS month,
       t.tax_id,
       SUM(CASE WHEN tx.transaction_type='RECEIPT' THEN l.tax_amount_minor ELSE 0 END) / 100.0 AS tax_paid,
       SUM(CASE WHEN tx.transaction_type='SHIPMENT' THEN l.tax_amount_minor ELSE 0 END) / 100.0 AS tax_collected
FROM commerce_purchase_order_lines l
JOIN commerce_taxes t ON t.tax_id = l.tax_id
JOIN commerce_inventory_transactions tx ON tx.reference_id = l.purchase_order_id
GROUP BY month, t.tax_id;
```

### 9.6 Top customers

```sql
SELECT c.id, c.name, c.contact_type,
       COUNT(DISTINCT so.id) AS order_count,
       SUM(so.total_minor) / 100.0 AS lifetime_revenue
FROM crm_customers c
JOIN commerce_sales_orders so ON so.customer_id = c.id
WHERE so.status IN ('INVOICED','PAID')
GROUP BY c.id
ORDER BY lifetime_revenue DESC
LIMIT 50;
```

### 9.7 Profit & loss (sales - cogs)

```sql
SELECT strftime('%Y-%m', so.created_at) AS month,
       SUM(so.total_minor - so.tax_total_minor) / 100.0 AS revenue_excl_tax,
       SUM(sl.quantity * p.cost_minor) / 100.0 AS cogs,
       (SUM(so.total_minor - so.tax_total_minor) - SUM(sl.quantity * p.cost_minor)) / 100.0 AS gross_profit
FROM commerce_sales_orders so
JOIN commerce_sales_lines sl ON sl.sales_order_id = so.id
JOIN commerce_products p ON p.id = sl.product_id
WHERE so.status IN ('INVOICED','PAID')
GROUP BY month
ORDER BY month DESC;
```

---

## 10. End-to-end example

Walk one laptop from purchase to paid invoice:

1. **Master data**: create `commerce_products` row `SKU-LAPTOP-001` (rate 1,500,000.00 TZS = 150,000,000 minor).
2. **Procurement**: create PR, approve, generate PO #PO-AAA against Acme Supplies for 10 units @ 1,200,000.00 each (120,000,000 minor).
3. **Receipt**: warehouse receives 10 laptops → `commerce_goods_receipts` + lines, ledger `+10` at Main Warehouse.
4. **CRM lead → customer**: salesperson wins lead, converts to customer, sends quote for 2 laptops.
5. **Quote accepted**: `quote_accept()` creates `commerce_sales_orders` (sub_total=300,000,000 minor, tax=54,000,000 minor, total=354,000,000 minor) and `accounting_invoices` draft.
6. **Shipment**: warehouse ships 2 laptops → `commerce_shipments` + ledger `-2`.
7. **Invoice sent**: invoice status -> `SENT`, due in 30 days.
8. **Payment received**: customer pays 354,000.00 TZS → `commerce_payments` + `commerce_payment_applications`; invoice status -> `PAID`, balance -> 0.
9. **Reports**: stock on hand = 8, lifetime revenue for this customer += 354,000, tax collected += 54,000.