// ERP module — was `models::commerce`.
// Renamed to ERP. Table names in SQL stay as `commerce_*` (no migration change),
// only the Rust module name changed.
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use crate::schema::*;

fn de_naive<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where D: Deserializer<'de> {
    let s = String::deserialize(d)?;
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) { return Ok(dt.naive_utc()); }
    for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%d %H:%M:%S", "%Y-%m-%d"] {
        if let Ok(nd) = NaiveDateTime::parse_from_str(&s, fmt) { return Ok(nd); }
    }
    let t = s.trim_end_matches('Z');
    for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S"] {
        if let Ok(nd) = NaiveDateTime::parse_from_str(t, fmt) { return Ok(nd); }
    }
    Err(serde::de::Error::custom(format!("invalid datetime '{}'", s)))
}
fn de_opt_naive<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
where D: Deserializer<'de> {
    let opt = Option::<String>::deserialize(d)?;
    match opt {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) { return Ok(Some(dt.naive_utc())); }
            for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%d %H:%M:%S", "%Y-%m-%d"] {
                if let Ok(nd) = NaiveDateTime::parse_from_str(&s, fmt) { return Ok(Some(nd)); }
            }
            let t = s.trim_end_matches('Z');
            for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S"] {
                if let Ok(nd) = NaiveDateTime::parse_from_str(t, fmt) { return Ok(Some(nd)); }
            }
            Err(serde::de::Error::custom(format!("invalid datetime '{}'", s)))
        }
    }
}

// ------------------------------------------------------------------- units --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_units)]
pub struct Unit { pub id: String, pub name: String, pub symbol: String, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = commerce_units)]
pub struct NewUnit { pub id: String, pub name: String, pub symbol: String }

// ---------------------------------------------------------------- warehouses --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_warehouses)]
pub struct Warehouse { pub id: String, pub name: String, pub location: Option<String>, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize, AsChangeset)]
#[diesel(table_name = commerce_warehouses)]
pub struct NewWarehouse { pub id: String, pub name: String, pub location: Option<String> }

// ---------------------------------------------------------------- suppliers --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_suppliers)]
pub struct Supplier { pub id: String, pub name: String, pub contact_email: Option<String>, pub contact_phone: Option<String>, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_suppliers)]
pub struct NewSupplier { pub id: String, pub name: String, pub contact_email: Option<String>, pub contact_phone: Option<String> }

// ---------------------------------------------------------------- products --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_products)]
pub struct Product { pub id: String, pub sku: String, pub name: String, pub description: Option<String>, pub unit_id: Option<String>, pub product_type: String, pub is_active: bool, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_products)]
pub struct NewProduct { pub id: String, pub sku: String, pub name: String, pub description: Option<String>, pub unit_id: Option<String>, pub product_type: String, pub is_active: bool }
#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = commerce_products)]
pub struct UpdateProduct { pub sku: Option<String>, pub name: Option<String>, pub description: Option<String>, pub unit_id: Option<String>, pub product_type: Option<String>, pub is_active: Option<bool> }

// ---------------------------------------------------------------- procurement --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_procurement_requests)]
pub struct ProcurementRequest { pub id: String, pub request_number: String, pub requested_by: Option<String>, pub status: String, pub notes: Option<String>, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_procurement_requests)]
pub struct NewProcurement { pub id: String, pub request_number: String, pub requested_by: Option<String>, pub status: String, pub notes: Option<String> }
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_procurement_lines)]
pub struct ProcurementLine { pub id: String, pub procurement_id: String, pub product_id: String, pub quantity: f64, pub notes: Option<String> }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_procurement_lines)]
pub struct NewProcurementLine { pub id: String, pub procurement_id: String, pub product_id: String, pub quantity: f64, pub notes: Option<String> }

// ---------------------------------------------------------------- purchasing --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_purchase_orders)]
pub struct PurchaseOrder { pub id: String, pub po_number: String, pub supplier_id: Option<String>, pub status: String, pub notes: Option<String>, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_purchase_orders)]
pub struct NewPurchaseOrder { pub id: String, pub po_number: String, pub supplier_id: Option<String>, pub status: String, pub notes: Option<String> }
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_purchase_order_lines)]
pub struct PurchaseOrderLine { pub id: String, pub purchase_order_id: String, pub product_id: String, pub quantity: f64, pub unit_price: f64, pub notes: Option<String> }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_purchase_order_lines)]
pub struct NewPurchaseOrderLine { pub id: String, pub purchase_order_id: String, pub product_id: String, pub quantity: f64, pub unit_price: f64, pub notes: Option<String> }

// ---------------------------------------------------------------- goods receipt --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_goods_receipts)]
pub struct GoodsReceipt { pub id: String, pub receipt_number: String, pub purchase_order_id: Option<String>, pub warehouse_id: Option<String>, pub received_at: NaiveDateTime, pub notes: Option<String>, pub created_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_goods_receipts)]
pub struct NewGoodsReceipt { pub id: String, pub receipt_number: String, pub purchase_order_id: Option<String>, pub warehouse_id: Option<String>, #[serde(deserialize_with = "de_naive")] pub received_at: NaiveDateTime, pub notes: Option<String> }
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_goods_receipt_lines)]
pub struct GoodsReceiptLine { pub id: String, pub receipt_id: String, pub product_id: String, pub quantity: f64, pub notes: Option<String> }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_goods_receipt_lines)]
pub struct NewGoodsReceiptLine { pub id: String, pub receipt_id: String, pub product_id: String, pub quantity: f64, pub notes: Option<String> }

// ---------------------------------------------------------------- inventory ledger --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_inventory_transactions)]
pub struct InventoryTransaction { pub id: String, pub product_id: String, pub warehouse_id: String, pub transaction_type: String, pub quantity: f64, pub reference_type: Option<String>, pub reference_id: Option<String>, pub created_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_inventory_transactions)]
pub struct NewInventoryTx { pub id: String, pub product_id: String, pub warehouse_id: String, pub transaction_type: String, pub quantity: f64, pub reference_type: Option<String>, pub reference_id: Option<String> }

// ---------------------------------------------------------------- sales --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_sales_orders)]
pub struct SalesOrder { pub id: String, pub so_number: String, pub customer_name: String, pub status: String, pub notes: Option<String>, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_sales_orders)]
pub struct NewSalesOrder { pub id: String, pub so_number: String, pub customer_name: String, pub status: String, pub notes: Option<String> }
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_sales_lines)]
pub struct SalesLine { pub id: String, pub sales_order_id: String, pub product_id: String, pub quantity: f64, pub unit_price: f64, pub notes: Option<String> }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_sales_lines)]
pub struct NewSalesLine { pub id: String, pub sales_order_id: String, pub product_id: String, pub quantity: f64, pub unit_price: f64, pub notes: Option<String> }
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_shipments)]
pub struct Shipment { pub id: String, pub sales_order_id: String, pub warehouse_id: Option<String>, pub shipped_at: NaiveDateTime, pub notes: Option<String> }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_shipments)]
pub struct NewShipment { pub id: String, pub sales_order_id: String, pub warehouse_id: Option<String>, #[serde(deserialize_with = "de_naive")] pub shipped_at: NaiveDateTime, pub notes: Option<String> }

// ---------------------------------------------------------------- assets --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_assets)]
pub struct Asset { pub id: String, pub asset_number: String, pub product_id: Option<String>, pub name: String, pub acquisition_cost: f64, pub acquisition_date: NaiveDateTime, pub status: String, pub location_id: Option<String>, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_assets)]
pub struct NewAsset { pub id: String, pub asset_number: String, pub product_id: Option<String>, pub name: String, pub acquisition_cost: f64, #[serde(deserialize_with = "de_naive")] pub acquisition_date: NaiveDateTime, pub status: String, pub location_id: Option<String> }

// ---------------------------------------------------------------- expenses --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = commerce_expenses)]
pub struct Expense { pub id: String, pub expense_number: String, pub category: String, pub description: String, pub amount: f64, pub expense_date: NaiveDateTime, pub status: String, pub approved_by: Option<String>, pub notes: Option<String>, pub created_at: NaiveDateTime, pub updated_at: NaiveDateTime }
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = commerce_expenses)]
pub struct NewExpense { pub id: String, pub expense_number: String, pub category: String, pub description: String, pub amount: f64, #[serde(deserialize_with = "de_naive")] pub expense_date: NaiveDateTime, pub status: String, pub approved_by: Option<String>, pub notes: Option<String> }