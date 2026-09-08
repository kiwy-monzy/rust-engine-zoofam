# CRM Module — Full Flow

The CRM module covers the full **lead-to-cash** lifecycle: marketing-sourced leads,
opportunity pipeline, customer 360, quotes, sales-order and invoice handoff into
Commerce, payments, support tickets, and reporting.

---

## 1. Master data setup

| Entity | Purpose |
|---|---|
| `gateway_organizations` | Multi-tenant root (already seeded by Commerce migration) |
| `crm_customers` | **Contact 360 root** — every person or company the org deals with |
| `crm_customer_persons` | Sub-contacts (multiple per customer) |
| `crm_customer_addresses` | Billing / shipping / other addresses (multiple per customer) |
| `crm_activities` | Tasks, calls, meetings, notes — linked to any pipeline entity |
| `crm_tickets` | Customer support cases |

`crm_customers` is enriched with the Zoho-style contact-360 fields:
`contact_type` (`customer | vendor | both`), `customer_sub_type` (`individual | business`),
`company_name`, `salutation`, `first_name`, `last_name`, `email`, `phone`, `mobile`,
`website`, `tax_id`, `tax_reg_no`, `currency_code`, `payment_terms`, `language_code`,
`outstanding_receivable_minor`, `unused_credits_minor`, `payment_reminder_enabled`,
`is_portal_enabled`, `portal_status`, `owner_id`, `status`, `notes`, and links to
the originating `lead_id` / `opportunity_id`.

---

## 2. Lead-to-cash pipeline

```
                    +------------------+
                    |  Marketing/Sales |
                    +--------+---------+
                             |
                             v
                    crm_leads        (NEW -> CONTACTED -> QUALIFIED)
                        |             \-> LOST
                        |              \-> WON (convert)
                        v
                    crm_opportunities (PROSPECT -> QUALIFIED -> PROPOSAL
                        |               -> NEGOTIATION -> WON/LOST)
                        |  amount_minor, currency_code, probability
                        v
                    crm_customers    (created from WON opp)
                        |  one-to-many
                        |-------- crm_customer_persons
                        |-------- crm_customer_addresses
                        |-------- crm_activities
                        v
                    crm_quotes       (DRAFT -> SENT -> ACCEPTED/REJECTED/EXPIRED)
                        |  one-to-many
                        v
                    crm_quote_lines  (product_id, qty, unit_price_minor, tax_*, line_total_minor)
                        |  (accept action)
                        v
                    commerce_sales_orders + commerce_sales_lines   <-- bridges to ERP
                        |
                        v
                    accounting_invoices  (DRAFT -> SENT -> PARTIAL/PAID/OVERDUE)
                        |
                        v
                    commerce_payments + commerce_payment_applications
```

The `converted_from_id` field on every doc (`crm_quotes`, `commerce_sales_orders`,
`accounting_invoices`) keeps the full chain auditable:
`quote -> sales_order -> invoice`.

---

## 3. Lead management (`crm_leads`)

Captured from a web form, an ad, a referral, or manually:

| Field | Purpose |
|---|---|
| `lead_number` | Unique per org, auto-generated like `LD-XXXXXXXX` |
| `company_name` / `contact_name` | At least one is required |
| `email`, `phone` | For follow-up |
| `source` | `web`, `referral`, `campaign`, `event`, ... |
| `status` | `NEW \| CONTACTED \| QUALIFIED \| LOST \| WON` |
| `assigned_to` | Salesperson user id |
| `product_id` | The product the lead is interested in |
| `estimated_value_minor` | Expected deal size in cents |

**Activity logging** on a lead:
```
crm_activities { related_type='lead', related_id=lead.id,
                 activity_type='call' | 'email' | 'meeting' | 'task' | 'note',
                 subject, due_at, done, notes }
```

When the lead is **qualified**, it converts to an **opportunity** (the lead row is
closed and the opportunity inherits the `company_name`, `contact_name`, etc.).

---

## 4. Opportunity pipeline (`crm_opportunities`)

| Field | Purpose |
|---|---|
| `opp_number` | Unique per org, `OPP-XXXXXXXX` |
| `lead_id` | Origin lead (nullable — some opps come in cold) |
| `customer_id` | Set if there is already an existing customer |
| `stage` | `PROSPECT \| QUALIFIED \| PROPOSAL \| NEGOTIATION \| WON \| LOST` |
| `probability` | 0.0 - 1.0, used for forecasting |
| `amount_minor` | Expected revenue (cents) |
| `currency_code` | For multi-currency forecasting |
| `expected_close` | Target date |

**Win:** on `stage = WON`, a `crm_customers` row is created (or matched) and
`opportunity.customer_id` is set.

---

## 5. Customer 360 (`crm_customers` + related tables)

The customer is the center of gravity. Every other doc references it.

### Sub-contacts (`crm_customer_persons`)

A company can have many people you actually deal with: the CFO who signs, the buyer
who places orders, the AP clerk who pays, the technical lead who uses the product.

```
crm_customer_persons {
    contact_person_id, customer_id, salutation, first_name, last_name,
    email, phone, mobile, designation, department, skype,
    is_primary, is_portal_enabled, sort_order
}
```

### Addresses (`crm_customer_addresses`)

Multiple per customer, with `kind = billing | shipping | other`:

```
crm_customer_addresses {
    address_id, customer_id, kind, attention, country, street, street2,
    city, state, zip, phone, fax, is_default
}
```

### Activity timeline (`crm_activities`)

Everything that happens with a customer lands on the timeline:
- `related_type = 'customer'`, `related_id = customer.id`
- Plus linked activities from their leads / opportunities / quotes

### Customer portal

If `is_portal_enabled = 1`, the customer gets a login and can:
- View their quotes, sales orders, invoices
- Pay invoices online (links to `commerce_payments`)
- Open and reply to support tickets (`crm_tickets`)

---

## 6. Quoting (`crm_quotes` + `crm_quote_lines`)

```
crm_quotes
    quote_number, customer_id, opportunity_id,
    status = DRAFT | SENT | ACCEPTED | REJECTED | EXPIRED,
    valid_until, currency_code, exchange_rate,
    sub_total_minor, discount_minor, tax_total_minor, total_minor,
    notes, terms,
    sales_order_id (set on accept), converted_from_id

crm_quote_lines
    quote_id, product_id, quantity,
    unit_price_minor, discount_minor,
    tax_id, tax_percentage, tax_amount_minor, line_total_minor,
    notes
```

**Money math per line:**
```
line_total_minor = quantity * unit_price_minor - discount_minor + tax_amount_minor
```
where `tax_amount_minor = round(line_subtotal * tax_percentage / 100)`.

**On accept** the controller `quote_accept()`:
1. Re-checks the quote has lines and is not already accepted.
2. Creates a `commerce_sales_orders` row with `converted_from_id = quote.id`.
3. Copies each `crm_quote_line` into a `commerce_sales_line` (with the same
   `unit_price_minor`, `tax_id`, `tax_percentage`).
4. Sets `crm_quotes.sales_order_id` and flips status to `ACCEPTED`.
5. Creates a draft `accounting_invoices` row with `converted_from_id = sales_order.id`.

---

## 7. Order → invoice handoff

After the ERP side runs (warehouse picks → shipment → ledger `-qty`), the
`accounting_invoices` row goes from `DRAFT` to `SENT` (and gets `invoice_date`,
`due_date`).

```
accounting_invoices
    invoice_number, customer_id, sales_order_id, quote_id,
    invoice_date, due_date,
    currency_code, exchange_rate,
    sub_total_minor, discount_minor, tax_total_minor, total_minor,
    payment_made_minor, credits_applied_minor, write_off_minor, balance_minor,
    status = DRAFT | SENT | PARTIAL | PAID | OVERDUE | WRITTEN_OFF | VOID,
    allow_partial_payments, notes, converted_from_id
```

`balance_minor = total_minor - payment_made_minor - credits_applied_minor - write_off_minor`.

---

## 8. Payments & customer balance

When money arrives:

```
commerce_payments
    payment_number, customer_id, payment_mode (cash | bank | mobile_money | card | cheque),
    payment_date, amount_minor, currency_code, exchange_rate,
    unused_minor, refunded_minor, reference_number, description

commerce_payment_applications
    payment_id, invoice_id, sales_order_id, amount_applied_minor
```

A single payment can be **split across multiple invoices** (or partially
unapplied — `unused_minor` is the residual). The customer's
`outstanding_receivable_minor` is recomputed as
`SUM(accounting_invoices.balance_minor WHERE customer_id = ? AND currency_code = ?)`.

---

## 9. Support tickets (`crm_tickets`)

```
crm_tickets
    ticket_number, customer_id, subject, body,
    status = open | pending | resolved | closed,
    priority = low | normal | high | urgent,
    assigned_to, created_at, updated_at
```

Tickets link to the customer (so the timeline shows the full context) and
optionally to a specific user (the `assigned_to` rep).

---

## 10. Reporting & analytics

### 10.1 Sales pipeline by stage

```sql
SELECT stage,
       COUNT(*) AS deals,
       SUM(amount_minor) / 100.0 AS pipeline_value,
       SUM(amount_minor * probability) / 100.0 AS weighted_value
FROM crm_opportunities
WHERE stage NOT IN ('WON','LOST')
GROUP BY stage
ORDER BY weighted_value DESC;
```

### 10.2 Win rate by source

```sql
SELECT l.source,
       COUNT(*) AS total_leads,
       SUM(CASE WHEN l.status='WON' THEN 1 ELSE 0 END) AS wins,
       1.0 * SUM(CASE WHEN l.status='WON' THEN 1 ELSE 0 END) / COUNT(*) AS win_rate
FROM crm_leads l
GROUP BY l.source
ORDER BY win_rate DESC;
```

### 10.3 Quote-to-close time

```sql
SELECT q.quote_number,
       julianday(i.invoice_date) - julianday(q.created_at) AS days_to_close
FROM crm_quotes q
JOIN accounting_invoices i ON i.quote_id = q.id
WHERE q.status = 'ACCEPTED';
```

### 10.4 Customer lifetime revenue

```sql
SELECT c.id, c.name, c.contact_type, c.currency_code,
       COUNT(DISTINCT so.id) AS orders,
       SUM(so.total_minor) / 100.0 AS lifetime_revenue,
       SUM(i.balance_minor) / 100.0 AS outstanding
FROM crm_customers c
LEFT JOIN commerce_sales_orders so ON so.customer_id = c.id
LEFT JOIN accounting_invoices i ON i.customer_id = c.id
GROUP BY c.id
ORDER BY lifetime_revenue DESC;
```

### 10.5 Receivables aging

```sql
SELECT c.id, c.name,
       SUM(i.balance_minor) / 100.0 AS total_outstanding,
       SUM(CASE WHEN i.due_date >= date('now') THEN i.balance_minor ELSE 0 END) / 100.0 AS current,
       SUM(CASE WHEN i.due_date <  date('now') AND julianday('now') - julianday(i.due_date) <= 30 THEN i.balance_minor ELSE 0 END) / 100.0 AS days_1_30,
       SUM(CASE WHEN julianday('now') - julianday(i.due_date) BETWEEN 31 AND 60  THEN i.balance_minor ELSE 0 END) / 100.0 AS days_31_60,
       SUM(CASE WHEN julianday('now') - julianday(i.due_date) >  60                THEN i.balance_minor ELSE 0 END) / 100.0 AS days_60_plus
FROM accounting_invoices i
JOIN crm_customers c ON c.id = i.customer_id
WHERE i.status NOT IN ('PAID','VOID')
GROUP BY c.id
ORDER BY total_outstanding DESC;
```

### 10.6 Top performers (sales reps)

```sql
SELECT c.owner_id,
       COUNT(DISTINCT c.id) AS customers,
       COUNT(DISTINCT so.id) AS orders,
       SUM(so.total_minor) / 100.0 AS revenue
FROM crm_customers c
LEFT JOIN commerce_sales_orders so ON so.customer_id = c.id
WHERE c.owner_id IS NOT NULL
GROUP BY c.owner_id
ORDER BY revenue DESC;
```

### 10.7 Lead conversion funnel

```sql
SELECT 'leads'        AS step, COUNT(*) FROM crm_leads
UNION ALL
SELECT 'qualified'    AS step, COUNT(*) FROM crm_leads WHERE status IN ('QUALIFIED','WON')
UNION ALL
SELECT 'opps'         AS step, COUNT(*) FROM crm_opportunities
UNION ALL
SELECT 'won_opps'     AS step, COUNT(*) FROM crm_opportunities WHERE stage='WON'
UNION ALL
SELECT 'customers'    AS step, COUNT(*) FROM crm_customers
UNION ALL
SELECT 'paid_invoices' AS step, COUNT(*) FROM accounting_invoices WHERE status='PAID';
```

### 10.8 Activity velocity

```sql
SELECT related_type,
       AVG(1.0 * (julianday(due_at) - julianday(created_at))) AS avg_days_to_due,
       AVG(CASE WHEN done = 1 THEN 1.0 ELSE 0 END) AS completion_rate
FROM crm_activities
WHERE due_at IS NOT NULL
GROUP BY related_type;
```

---

## 11. End-to-end example

Walk one laptop from a marketing lead to a paid invoice:

1. **Lead**: visitor fills out a web form → `crm_leads { source='web', status='NEW' }`.
2. **Qualify**: rep calls, sends brochure → `crm_activities { type='call' }`, status -> `QUALIFIED`.
3. **Opportunity**: rep creates `crm_opportunities { stage='PROSPECT', amount_minor=300_000_00, probability=0.3 }`.
4. **Customer**: on `WON`, a `crm_customers` row is created (id + number) and linked.
5. **Person + address**: rep adds the buyer's email (`crm_customer_persons`) and the shipping address (`crm_customer_addresses kind='shipping'`).
6. **Quote**: rep sends a `crm_quotes` for 2 laptops (sub_total=300,000,000 minor, VAT 18% = 54,000,000 minor, total=354,000,000 minor).
7. **Accept**: customer clicks accept → `quote_accept()` creates `commerce_sales_orders` + lines + draft `accounting_invoices`, all linked by `converted_from_id`.
8. **Ship**: warehouse ships 2 laptops → ledger `-2`, shipment `status='delivered'`.
9. **Invoice**: invoice status `SENT`, `due_date = today + 30`.
10. **Pay**: customer pays 354,000 TZS → `commerce_payments` + applications, invoice `status='PAID'`, `balance_minor=0`.
11. **Report**: customer lifetime revenue += 354,000; rep dashboard updates; aging report shows nothing outstanding.