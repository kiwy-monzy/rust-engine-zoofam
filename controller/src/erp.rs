// ERP controller — was `controller::commerce`.
// Renamed to ERP. Function names and behavior unchanged.
use chrono::{NaiveDateTime, Utc};
use db::DbPool;
use diesel::prelude::*;
use uuid::Uuid;
use crate::Error;
use models::erp::*;
use models::schema::*;

fn now() -> NaiveDateTime { Utc::now().naive_utc() }
fn nid() -> String { Uuid::new_v4().to_string() }

pub fn products(pool: &DbPool) -> Result<Vec<Product>, Error> {
    let mut c = db::conn(pool)?; Ok(commerce_products::table.order(commerce_products::created_at.desc()).load(&mut c)?)
}
pub fn product_create(pool: &DbPool, mut v: NewProduct) -> Result<Product, Error> {
    if v.id.is_empty() { v.id = nid(); } let mut c = db::conn(pool)?; diesel::insert_into(commerce_products::table).values(&v).execute(&mut c)?; Ok(commerce_products::table.find(&v.id).first(&mut c)?)
}
pub fn product_update(pool: &DbPool, id:&str, patch: UpdateProduct) -> Result<Product, Error> {
    let mut c = db::conn(pool)?; diesel::update(commerce_products::table.find(id)).set((patch, commerce_products::updated_at.eq(now()))).execute(&mut c)?; Ok(commerce_products::table.find(id).first(&mut c)?)
}
pub fn product_delete(pool: &DbPool, id:&str)->Result<(),Error>{ let mut c=db::conn(pool)?; diesel::delete(commerce_products::table.find(id)).execute(&mut c)?; Ok(()) }

pub fn warehouses(pool:&DbPool)->Result<Vec<Warehouse>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_warehouses::table.load(&mut c)?) }
pub fn warehouse_create(pool:&DbPool, mut v: NewWarehouse)->Result<Warehouse,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(commerce_warehouses::table).values(&v).execute(&mut c)?; Ok(commerce_warehouses::table.find(&v.id).first(&mut c)?) }
pub fn units(pool:&DbPool)->Result<Vec<Unit>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_units::table.load(&mut c)?) }
pub fn unit_create(pool:&DbPool, mut v: NewUnit)->Result<Unit,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(commerce_units::table).values(&v).execute(&mut c)?; Ok(commerce_units::table.find(&v.id).first(&mut c)?) }
pub fn suppliers(pool:&DbPool)->Result<Vec<Supplier>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_suppliers::table.load(&mut c)?) }
pub fn supplier_create(pool:&DbPool, mut v: NewSupplier)->Result<Supplier,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(commerce_suppliers::table).values(&v).execute(&mut c)?; Ok(commerce_suppliers::table.find(&v.id).first(&mut c)?) }

pub fn procurement(pool:&DbPool)->Result<Vec<ProcurementRequest>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_procurement_requests::table.order(commerce_procurement_requests::created_at.desc()).load(&mut c)?) }
pub fn procurement_create(pool:&DbPool, mut v: NewProcurement)->Result<ProcurementRequest,Error>{ if v.id.is_empty(){v.id=nid();} if v.request_number.is_empty(){v.request_number = format!("PR-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(commerce_procurement_requests::table).values(&v).execute(&mut c)?; Ok(commerce_procurement_requests::table.find(&v.id).first(&mut c)?) }

pub fn purchase_orders(pool:&DbPool)->Result<Vec<PurchaseOrder>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_purchase_orders::table.order(commerce_purchase_orders::created_at.desc()).load(&mut c)?) }
pub fn po_create(pool:&DbPool, mut v: NewPurchaseOrder)->Result<PurchaseOrder,Error>{ if v.id.is_empty(){v.id=nid();} if v.po_number.is_empty(){v.po_number = format!("PO-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(commerce_purchase_orders::table).values(&v).execute(&mut c)?; Ok(commerce_purchase_orders::table.find(&v.id).first(&mut c)?) }
pub fn po_lines(pool:&DbPool, po:&str)->Result<Vec<PurchaseOrderLine>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_purchase_order_lines::table.filter(commerce_purchase_order_lines::purchase_order_id.eq(po)).load(&mut c)?) }
pub fn po_line_create(pool:&DbPool, mut v: NewPurchaseOrderLine)->Result<PurchaseOrderLine,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(commerce_purchase_order_lines::table).values(&v).execute(&mut c)?; Ok(commerce_purchase_order_lines::table.find(&v.id).first(&mut c)?) }

pub fn receipts(pool:&DbPool)->Result<Vec<GoodsReceipt>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_goods_receipts::table.order(commerce_goods_receipts::created_at.desc()).load(&mut c)?) }
pub fn receipt_create(pool:&DbPool, mut v: NewGoodsReceipt, lines: Vec<NewGoodsReceiptLine>)->Result<GoodsReceipt,Error>{
    if v.id.is_empty(){v.id=nid();} if v.receipt_number.is_empty(){v.receipt_number=format!("GR-{}", &v.id[..8]);}
    let mut c=db::conn(pool)?; c.transaction(|conn|{
        diesel::insert_into(commerce_goods_receipts::table).values(&v).execute(conn)?;
        for mut l in lines { if l.id.is_empty(){l.id=nid();} l.receipt_id=v.id.clone(); diesel::insert_into(commerce_goods_receipt_lines::table).values(&l).execute(conn)?;
            let tx = NewInventoryTx{ id:nid(), product_id:l.product_id.clone(), warehouse_id: v.warehouse_id.clone().unwrap_or_else(|| "00000000-0000-0000-0000-000000000011".into()), transaction_type:"RECEIPT".into(), quantity: l.quantity, reference_type:Some("GOODS_RECEIPT".into()), reference_id:Some(v.id.clone()) };
            diesel::insert_into(commerce_inventory_transactions::table).values(&tx).execute(conn)?;
        }
        Ok::<_, diesel::result::Error>(())
    })?; let mut c2=db::conn(pool)?; Ok(commerce_goods_receipts::table.find(&v.id).first(&mut c2)?)
}

pub fn inventory_txs(pool:&DbPool)->Result<Vec<InventoryTransaction>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_inventory_transactions::table.order(commerce_inventory_transactions::created_at.desc()).limit(200).load(&mut c)?) }
pub fn inventory_stock(pool:&DbPool)->Result<Vec<(String,String,f64)>,Error>{
    let txs = inventory_txs(pool)?; use std::collections::HashMap; let mut m:HashMap<(String,String),f64>=HashMap::new(); for t in txs { *m.entry((t.product_id,t.warehouse_id)).or_default()+=t.quantity; } Ok(m.into_iter().map(|((p,w),q)|(p,w,q)).collect())
}

pub fn sales_orders(pool:&DbPool)->Result<Vec<SalesOrder>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_sales_orders::table.order(commerce_sales_orders::created_at.desc()).load(&mut c)?) }
pub fn sales_create(pool:&DbPool, mut v: NewSalesOrder)->Result<SalesOrder,Error>{ if v.id.is_empty(){v.id=nid();} if v.so_number.is_empty(){v.so_number=format!("SO-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(commerce_sales_orders::table).values(&v).execute(&mut c)?; Ok(commerce_sales_orders::table.find(&v.id).first(&mut c)?) }
pub fn sales_lines(pool:&DbPool, so:&str)->Result<Vec<SalesLine>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_sales_lines::table.filter(commerce_sales_lines::sales_order_id.eq(so)).load(&mut c)?) }
pub fn sales_line_create(pool:&DbPool, mut v: NewSalesLine)->Result<SalesLine,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(commerce_sales_lines::table).values(&v).execute(&mut c)?; Ok(commerce_sales_lines::table.find(&v.id).first(&mut c)?) }
pub fn shipment_create(pool:&DbPool, mut v: NewShipment, lines: Vec<NewSalesLine>)->Result<Shipment,Error>{
    if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; c.transaction(|conn|{
        diesel::insert_into(commerce_shipments::table).values(&v).execute(conn)?;
        for l in &lines {
            let tx = NewInventoryTx{ id:nid(), product_id:l.product_id.clone(), warehouse_id: v.warehouse_id.clone().unwrap_or_else(|| "00000000-0000-0000-0000-000000000011".into()), transaction_type:"SHIPMENT".into(), quantity: -l.quantity, reference_type:Some("SALES_ORDER".into()), reference_id:Some(v.sales_order_id.clone()) };
            diesel::insert_into(commerce_inventory_transactions::table).values(&tx).execute(conn)?;
        }
        Ok::<_, diesel::result::Error>(())
    })?; let mut c2=db::conn(pool)?; Ok(commerce_shipments::table.find(&v.id).first(&mut c2)?)
}

pub fn assets(pool:&DbPool)->Result<Vec<Asset>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_assets::table.order(commerce_assets::created_at.desc()).load(&mut c)?) }
pub fn asset_create(pool:&DbPool, mut v: NewAsset)->Result<Asset,Error>{
    if v.id.is_empty(){v.id=nid();} if v.asset_number.is_empty(){v.asset_number=format!("AST-{}", &v.id[..8]);}
    let mut c=db::conn(pool)?; c.transaction(|conn|{
        diesel::insert_into(commerce_assets::table).values(&v).execute(conn)?;
        if let Some(pid)=&v.product_id { let tx=NewInventoryTx{ id:nid(), product_id: pid.clone(), warehouse_id: v.location_id.clone().unwrap_or_else(|| "00000000-0000-0000-0000-000000000011".into()), transaction_type:"ASSET_CAPITALIZATION".into(), quantity: -1.0, reference_type:Some("ASSET".into()), reference_id:Some(v.id.clone()) }; diesel::insert_into(commerce_inventory_transactions::table).values(&tx).execute(conn)?; }
        Ok::<_, diesel::result::Error>(())
    })?; let mut c2=db::conn(pool)?; Ok(commerce_assets::table.find(&v.id).first(&mut c2)?)
}

pub fn expenses(pool:&DbPool)->Result<Vec<Expense>,Error>{ let mut c=db::conn(pool)?; Ok(commerce_expenses::table.order(commerce_expenses::created_at.desc()).load(&mut c)?) }
pub fn expense_create(pool:&DbPool, mut v: NewExpense)->Result<Expense,Error>{ if v.id.is_empty(){v.id=nid();} if v.expense_number.is_empty(){v.expense_number=format!("EXP-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(commerce_expenses::table).values(&v).execute(&mut c)?; Ok(commerce_expenses::table.find(&v.id).first(&mut c)?) }
pub fn expense_delete(pool:&DbPool, id:&str)->Result<(),Error>{ let mut c=db::conn(pool)?; diesel::delete(commerce_expenses::table.find(id)).execute(&mut c)?; Ok(()) }