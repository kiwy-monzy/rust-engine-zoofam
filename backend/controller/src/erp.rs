// ERP controller — was `controller::commerce`.
// Renamed to ERP. Function names and behavior unchanged.
use crate::Error;
use chrono::{NaiveDateTime, Utc};
use db::DbPool;
use diesel::prelude::*;
use models::erp::*;
use models::schema::*;
use uuid::Uuid;

fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}
fn nid() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StockLevel {
    pub product_id: String,
    pub warehouse_id: String,
    pub quantity: f64,
}

pub fn products(pool: &DbPool) -> Result<Vec<Product>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_products::table
        .order(erp_products::created_at.desc())
        .select(Product::as_select())
        .load(&mut c)?)
}
pub fn product_create(pool: &DbPool, mut v: NewProduct) -> Result<Product, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_products::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_products::table.find(&v.id).select(Product::as_select()).first(&mut c)?)
}
pub fn product_update(pool: &DbPool, id: &str, patch: UpdateProduct) -> Result<Product, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_products::table.find(id))
        .set((patch, erp_products::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_products::table.find(id).select(Product::as_select()).first(&mut c)?)
}
pub fn product_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_products::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn warehouses(pool: &DbPool) -> Result<Vec<Warehouse>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_warehouses::table.select(Warehouse::as_select()).load(&mut c)?)
}
pub fn warehouse_create(pool: &DbPool, mut v: NewWarehouse) -> Result<Warehouse, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_warehouses::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_warehouses::table.find(&v.id).select(Warehouse::as_select()).first(&mut c)?)
}
pub fn units(pool: &DbPool) -> Result<Vec<Unit>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_units::table.select(Unit::as_select()).load(&mut c)?)
}
pub fn unit_create(pool: &DbPool, mut v: NewUnit) -> Result<Unit, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_units::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_units::table.find(&v.id).select(Unit::as_select()).first(&mut c)?)
}
pub fn suppliers(pool: &DbPool) -> Result<Vec<Supplier>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_suppliers::table.select(Supplier::as_select()).load(&mut c)?)
}
pub fn supplier_create(pool: &DbPool, mut v: NewSupplier) -> Result<Supplier, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_suppliers::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_suppliers::table.find(&v.id).select(Supplier::as_select()).first(&mut c)?)
}

pub fn procurement(pool: &DbPool) -> Result<Vec<ProcurementRequest>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_procurement_requests::table
        .order(erp_procurement_requests::created_at.desc())
        .select(ProcurementRequest::as_select())
        .load(&mut c)?)
}
pub fn procurement_create(
    pool: &DbPool,
    mut v: NewProcurement,
) -> Result<ProcurementRequest, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    if v.request_number.is_empty() {
        v.request_number = format!("PR-{}", &v.id[..8]);
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_procurement_requests::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_procurement_requests::table.find(&v.id).select(ProcurementRequest::as_select()).first(&mut c)?)
}

pub fn purchase_orders(pool: &DbPool) -> Result<Vec<PurchaseOrder>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_purchase_orders::table
        .order(erp_purchase_orders::created_at.desc())
        .select(PurchaseOrder::as_select())
        .load(&mut c)?)
}
pub fn po_create(pool: &DbPool, mut v: NewPurchaseOrder) -> Result<PurchaseOrder, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    if v.po_number.is_empty() {
        v.po_number = format!("PO-{}", &v.id[..8]);
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_purchase_orders::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_purchase_orders::table.find(&v.id).select(PurchaseOrder::as_select()).first(&mut c)?)
}
pub fn po_lines(pool: &DbPool, po: &str) -> Result<Vec<PurchaseOrderLine>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_purchase_order_lines::table
        .filter(erp_purchase_order_lines::po_id.eq(po))
        .select(PurchaseOrderLine::as_select())
        .load(&mut c)?)
}
pub fn po_line_create(
    pool: &DbPool,
    mut v: NewPurchaseOrderLine,
) -> Result<PurchaseOrderLine, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_purchase_order_lines::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_purchase_order_lines::table.find(&v.id).select(PurchaseOrderLine::as_select()).first(&mut c)?)
}

pub fn receipts(pool: &DbPool) -> Result<Vec<GoodsReceipt>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_goods_receipts::table
        .order(erp_goods_receipts::created_at.desc())
        .select(GoodsReceipt::as_select())
        .load(&mut c)?)
}
pub fn receipt_create(
    pool: &DbPool,
    mut v: NewGoodsReceipt,
    lines: Vec<NewGoodsReceiptLine>,
) -> Result<GoodsReceipt, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    if v.receipt_number.is_empty() {
        v.receipt_number = format!("GR-{}", &v.id[..8]);
    }
    let mut c = db::conn(pool)?;
    c.transaction(|conn| {
        diesel::insert_into(erp_goods_receipts::table)
            .values(&v)
            .execute(conn)?;
        for mut l in lines {
            if l.id.is_empty() {
                l.id = nid();
            }
            l.receipt_id = v.id.clone();
            diesel::insert_into(erp_goods_receipt_lines::table)
                .values(&l)
                .execute(conn)?;
            let tx = NewInventoryTx {
                id: nid(),
                product_id: l.product_id.clone(),
                warehouse_id: v.warehouse_id.clone(),
                transaction_type: "RECEIPT".into(),
                quantity: l.quantity,
                reference_type: Some("GOODS_RECEIPT".into()),
                reference_id: Some(v.id.clone()),
            };
            diesel::insert_into(erp_inventory_transactions::table)
                .values(&tx)
                .execute(conn)?;
        }
        Ok::<_, diesel::result::Error>(())
    })?;
    let mut c2 = db::conn(pool)?;
    Ok(erp_goods_receipts::table.find(&v.id).select(GoodsReceipt::as_select()).first(&mut c2)?)
}

pub fn inventory_txs(pool: &DbPool) -> Result<Vec<InventoryTransaction>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_inventory_transactions::table
        .order(erp_inventory_transactions::created_at.desc())
        .select(InventoryTransaction::as_select())
        .limit(200)
        .load(&mut c)?)
}
pub fn inventory_stock(pool: &DbPool) -> Result<Vec<StockLevel>, Error> {
    let txs = inventory_txs(pool)?;
    use std::collections::HashMap;
    let mut m: HashMap<(String, String), f64> = HashMap::new();
    for t in txs {
        *m.entry((t.product_id, t.warehouse_id)).or_default() += t.quantity as f64;
    }
    Ok(m.into_iter()
        .map(|((p, w), q)| StockLevel {
            product_id: p,
            warehouse_id: w,
            quantity: q,
        })
        .collect())
}

pub fn sales_orders(pool: &DbPool) -> Result<Vec<SalesOrder>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_sales_orders::table
        .order(erp_sales_orders::created_at.desc())
        .select(SalesOrder::as_select())
        .load(&mut c)?)
}
pub fn sales_create(pool: &DbPool, mut v: NewSalesOrder) -> Result<SalesOrder, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    if v.so_number.is_empty() {
        v.so_number = format!("SO-{}", &v.id[..8]);
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_sales_orders::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_sales_orders::table.find(&v.id).select(SalesOrder::as_select()).first(&mut c)?)
}
pub fn sales_lines(pool: &DbPool, so: &str) -> Result<Vec<SalesLine>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_sales_lines::table
        .filter(erp_sales_lines::sales_order_id.eq(so))
        .select(SalesLine::as_select())
        .load(&mut c)?)
}
pub fn sales_line_create(pool: &DbPool, mut v: NewSalesLine) -> Result<SalesLine, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_sales_lines::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_sales_lines::table.find(&v.id).select(SalesLine::as_select()).first(&mut c)?)
}
pub fn shipment_create(
    pool: &DbPool,
    mut v: NewShipment,
    lines: Vec<NewSalesLine>,
) -> Result<Shipment, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    c.transaction(|conn| {
        diesel::insert_into(erp_shipments::table)
            .values(&v)
            .execute(conn)?;
        for l in &lines {
            let tx = NewInventoryTx {
                id: nid(),
                product_id: l.product_id.clone(),
                warehouse_id: v.warehouse_id.clone().unwrap_or_else(|| "00000000-0000-0000-0000-000000000011".into()),
                transaction_type: "SHIPMENT".into(),
                quantity: -l.quantity,
                reference_type: Some("SALES_ORDER".into()),
                reference_id: Some(v.sales_order_id.clone()),
            };
            diesel::insert_into(erp_inventory_transactions::table)
                .values(&tx)
                .execute(conn)?;
        }
        Ok::<_, diesel::result::Error>(())
    })?;
    let mut c2 = db::conn(pool)?;
    Ok(erp_shipments::table.find(&v.id).select(Shipment::as_select()).first(&mut c2)?)
}

pub fn assets(pool: &DbPool) -> Result<Vec<Asset>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_assets::table
        .order(erp_assets::created_at.desc())
        .select(Asset::as_select())
        .load(&mut c)?)
}
pub fn asset_create(pool: &DbPool, mut v: NewAsset) -> Result<Asset, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    if v.asset_number.is_empty() {
        v.asset_number = format!("AST-{}", &v.id[..8]);
    }
    let mut c = db::conn(pool)?;
    c.transaction(|conn| {
        diesel::insert_into(erp_assets::table)
            .values(&v)
            .execute(conn)?;
        if let Some(pid) = &v.product_id {
            let tx = NewInventoryTx {
                id: nid(),
                product_id: pid.clone(),
                warehouse_id: v.location_id.clone().unwrap_or_else(|| "00000000-0000-0000-0000-000000000011".into()),
                transaction_type: "ASSET_CAPITALIZATION".into(),
                quantity: -1.0,
                reference_type: Some("ASSET".into()),
                reference_id: Some(v.id.clone()),
            };
            diesel::insert_into(erp_inventory_transactions::table)
                .values(&tx)
                .execute(conn)?;
        }
        Ok::<_, diesel::result::Error>(())
    })?;
    let mut c2 = db::conn(pool)?;
    Ok(erp_assets::table.find(&v.id).select(Asset::as_select()).first(&mut c2)?)
}

pub fn expenses(pool: &DbPool) -> Result<Vec<Expense>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_expenses::table
        .order(erp_expenses::created_at.desc())
        .select(Expense::as_select())
        .load(&mut c)?)
}
pub fn expense_create(pool: &DbPool, mut v: NewExpense) -> Result<Expense, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    if v.expense_number.is_empty() {
        v.expense_number = format!("EXP-{}", &v.id[..8]);
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_expenses::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_expenses::table.find(&v.id).select(Expense::as_select()).first(&mut c)?)
}
pub fn expense_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_expenses::table.find(id)).execute(&mut c)?;
    Ok(())
}

// =====================================================
// Full CRUD additions (update / delete / list / get-one)
// =====================================================

pub fn unit_update(pool: &DbPool, id: &str, patch: UpdateUnit) -> Result<Unit, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_units::table.find(id))
        .set((patch, erp_units::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_units::table.find(id).select(Unit::as_select()).first(&mut c)?)
}
pub fn unit_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_units::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn warehouse_update(
    pool: &DbPool,
    id: &str,
    patch: UpdateWarehouse,
) -> Result<Warehouse, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_warehouses::table.find(id))
        .set((patch, erp_warehouses::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_warehouses::table.find(id).select(Warehouse::as_select()).first(&mut c)?)
}
pub fn warehouse_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_warehouses::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn supplier_update(pool: &DbPool, id: &str, patch: UpdateSupplier) -> Result<Supplier, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_suppliers::table.find(id))
        .set((patch, erp_suppliers::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_suppliers::table.find(id).select(Supplier::as_select()).first(&mut c)?)
}
pub fn supplier_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_suppliers::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn procurement_update(
    pool: &DbPool,
    id: &str,
    patch: UpdateProcurement,
) -> Result<ProcurementRequest, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_procurement_requests::table.find(id))
        .set((patch, erp_procurement_requests::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_procurement_requests::table.find(id).select(ProcurementRequest::as_select()).first(&mut c)?)
}
pub fn procurement_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_procurement_requests::table.find(id)).execute(&mut c)?;
    Ok(())
}
pub fn procurement_lines(pool: &DbPool, pr: &str) -> Result<Vec<ProcurementLine>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_procurement_lines::table
        .filter(erp_procurement_lines::request_id.eq(pr))
        .select(ProcurementLine::as_select())
        .load(&mut c)?)
}
pub fn procurement_line_create(
    pool: &DbPool,
    mut v: NewProcurementLine,
) -> Result<ProcurementLine, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_procurement_lines::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_procurement_lines::table.find(&v.id).select(ProcurementLine::as_select()).first(&mut c)?)
}

pub fn po_update(
    pool: &DbPool,
    id: &str,
    patch: UpdatePurchaseOrder,
) -> Result<PurchaseOrder, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_purchase_orders::table.find(id))
        .set((patch, erp_purchase_orders::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_purchase_orders::table.find(id).select(PurchaseOrder::as_select()).first(&mut c)?)
}
pub fn po_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_purchase_orders::table.find(id)).execute(&mut c)?;
    Ok(())
}
pub fn po_line_update(
    pool: &DbPool,
    id: &str,
    patch: UpdatePurchaseOrderLine,
) -> Result<PurchaseOrderLine, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_purchase_order_lines::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    Ok(erp_purchase_order_lines::table.find(id).select(PurchaseOrderLine::as_select()).first(&mut c)?)
}
pub fn po_line_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_purchase_order_lines::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn receipt_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_goods_receipts::table.find(id)).execute(&mut c)?;
    Ok(())
}
pub fn receipt_lines(pool: &DbPool, pr: &str) -> Result<Vec<GoodsReceiptLine>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_goods_receipt_lines::table
        .filter(erp_goods_receipt_lines::receipt_id.eq(pr))
        .select(GoodsReceiptLine::as_select())
        .load(&mut c)?)
}
pub fn receipt_line_create(
    pool: &DbPool,
    mut v: NewGoodsReceiptLine,
) -> Result<GoodsReceiptLine, Error> {
    if v.id.is_empty() {
        v.id = nid();
    }
    let mut c = db::conn(pool)?;
    diesel::insert_into(erp_goods_receipt_lines::table)
        .values(&v)
        .execute(&mut c)?;
    Ok(erp_goods_receipt_lines::table.find(&v.id).select(GoodsReceiptLine::as_select()).first(&mut c)?)
}

pub fn sales_update(pool: &DbPool, id: &str, patch: UpdateSalesOrder) -> Result<SalesOrder, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_sales_orders::table.find(id))
        .set((patch, erp_sales_orders::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_sales_orders::table.find(id).select(SalesOrder::as_select()).first(&mut c)?)
}
pub fn sales_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_sales_orders::table.find(id)).execute(&mut c)?;
    Ok(())
}
pub fn sales_line_update(
    pool: &DbPool,
    id: &str,
    patch: UpdateSalesLine,
) -> Result<SalesLine, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_sales_lines::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    Ok(erp_sales_lines::table.find(id).select(SalesLine::as_select()).first(&mut c)?)
}
pub fn sales_line_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_sales_lines::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn shipments(pool: &DbPool) -> Result<Vec<Shipment>, Error> {
    let mut c = db::conn(pool)?;
    Ok(erp_shipments::table
        .order(erp_shipments::shipped_at.desc())
        .select(Shipment::as_select())
        .load(&mut c)?)
}
pub fn shipment_update(pool: &DbPool, id: &str, patch: UpdateShipment) -> Result<Shipment, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_shipments::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    Ok(erp_shipments::table.find(id).select(Shipment::as_select()).first(&mut c)?)
}
pub fn shipment_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_shipments::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn asset_update(pool: &DbPool, id: &str, patch: UpdateAsset) -> Result<Asset, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_assets::table.find(id))
        .set((patch, erp_assets::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_assets::table.find(id).select(Asset::as_select()).first(&mut c)?)
}
pub fn asset_delete(pool: &DbPool, id: &str) -> Result<(), Error> {
    let mut c = db::conn(pool)?;
    diesel::delete(erp_assets::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn expense_update(pool: &DbPool, id: &str, patch: UpdateExpense) -> Result<Expense, Error> {
    let mut c = db::conn(pool)?;
    diesel::update(erp_expenses::table.find(id))
        .set((patch, erp_expenses::updated_at.eq(now())))
        .execute(&mut c)?;
    Ok(erp_expenses::table.find(id).select(Expense::as_select()).first(&mut c)?)
}
pub fn expenses_clear(pool: &DbPool) -> Result<usize, Error> {
    let mut c = db::conn(pool)?;
    Ok(diesel::delete(erp_expenses::table).execute(&mut c)?)
}
