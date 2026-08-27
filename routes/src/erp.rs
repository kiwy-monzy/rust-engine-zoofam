// ERP routes — all paths under `/erp/*` (renamed from `/commerce/*`).
// Full CRUD where applicable: products, units, warehouses, suppliers,
// procurement, purchase orders + lines, goods receipts + lines, inventory,
// sales orders + lines, shipments, assets, expenses.
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json, Router,
    routing::{delete, get, patch, post},
};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::{middleware::{ApiError, ApiResult}, state::AppState};
use auth::Claims;
use models::schema::*;

fn need(c: &Claims, m: &str, a: &str) -> Result<(), ApiError> {
    c.require(m, a).map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string()))
}

#[derive(Deserialize, Default)]
pub struct ListQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

pub fn erp_routes() -> Router<AppState> {
    Router::new()
        // ---------- Products ----------
        .route("/erp/products", get(list_products).post(create_product))
        .route("/erp/products/:id", get(get_product).patch(update_product).delete(delete_product))
        // ---------- Units ----------
        .route("/erp/units", get(list_units).post(create_unit))
        .route("/erp/units/:id", get(get_unit).patch(update_unit).delete(delete_unit))
        // ---------- Warehouses ----------
        .route("/erp/warehouses", get(list_warehouses).post(create_warehouse))
        .route("/erp/warehouses/:id", get(get_warehouse).patch(update_warehouse).delete(delete_warehouse))
        // ---------- Suppliers ----------
        .route("/erp/suppliers", get(list_suppliers).post(create_supplier))
        .route("/erp/suppliers/:id", get(get_supplier).patch(update_supplier).delete(delete_supplier))
        // ---------- Procurement ----------
        .route("/erp/procurement", get(list_procurement).post(create_procurement))
        .route("/erp/procurement/:id", get(get_procurement).patch(update_procurement).delete(delete_procurement))
        .route("/erp/procurement/:id/lines", get(list_procurement_lines).post(create_procurement_line))
        // ---------- Purchase Orders ----------
        .route("/erp/purchase-orders", get(list_pos).post(create_po))
        .route("/erp/purchase-orders/:id", get(get_po).patch(update_po).delete(delete_po))
        .route("/erp/purchase-orders/:id/lines", get(list_po_lines).post(create_po_line))
        .route("/erp/purchase-orders/:id/lines/:line_id", get(get_po_line).patch(update_po_line).delete(delete_po_line))
        // ---------- Goods Receipts ----------
        .route("/erp/goods-receipts", get(list_receipts).post(create_receipt))
        .route("/erp/goods-receipts/:id", get(get_receipt).delete(delete_receipt))
        .route("/erp/goods-receipts/:id/lines", get(list_receipt_lines).post(create_receipt_line))
        // ---------- Inventory ----------
        .route("/erp/inventory/transactions", get(list_inventory))
        .route("/erp/inventory/transactions/:id", get(get_transaction))
        .route("/erp/inventory/stock", get(stock))
        // ---------- Sales Orders ----------
        .route("/erp/sales-orders", get(list_sales).post(create_sales))
        .route("/erp/sales-orders/:id", get(get_sales).patch(update_sales).delete(delete_sales))
        .route("/erp/sales-orders/:id/lines", get(list_sales_lines).post(create_sales_line))
        .route("/erp/sales-orders/:id/lines/:line_id", get(get_sales_line).patch(update_sales_line).delete(delete_sales_line))
        // ---------- Shipments ----------
        .route("/erp/shipments", get(list_shipments).post(create_shipment))
        .route("/erp/shipments/:id", get(get_shipment).patch(update_shipment).delete(delete_shipment))
        // ---------- Assets ----------
        .route("/erp/assets", get(list_assets).post(create_asset))
        .route("/erp/assets/:id", get(get_asset).patch(update_asset).delete(delete_asset))
        // ---------- Expenses ----------
        .route("/erp/expenses", get(list_expenses).post(create_expense))
        .route("/erp/expenses/:id", get(get_expense).patch(update_expense).delete(delete_expense))
        .route("/erp/expenses/clear", delete(clear_expenses))
}

// ---- helpers ----
fn one<T: serde::Serialize>(
    s: &AppState, c: &Claims, table_singular: &str, id: &str,
    f: impl FnOnce(&mut db::DbConn) -> Result<T, ApiError>,
) -> ApiResult<Json<Value>> {
    need(c, "commerce", "read")?;
    let mut conn = db::conn(&s.pool).map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let v = f(&mut conn)?;
    Ok(Json(json!({ table_singular: v, "id": id })))
}

fn find_one<T, F>(table: &str, id: &str, f: F) -> ApiResult<Json<Value>>
where
    T: serde::Serialize,
    F: FnOnce() -> Result<T, ApiError>,
{
    match f() {
        Ok(v) => Ok(Json(json!({ table: v, "id": id }))),
        Err(_) => Err(ApiError::new(StatusCode::NOT_FOUND, format!("{} {} not found", table, id))),
    }
}

// ============================================================================
// Products
// ============================================================================
async fn list_products(State(s): State<AppState>, Extension(c): Extension<Claims>, Query(q): Query<ListQuery>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"products": controller::erp::products(&s.pool)?, "limit": q.limit, "offset": q.offset})))
}
async fn get_product(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "product", &id, |conn| {
        models::erp::Product::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_product(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewProduct>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let p = controller::erp::product_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"product": p}))))
}
async fn update_product(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::UpdateProduct>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"product": controller::erp::product_update(&s.pool, &id, v)?})))
}
async fn delete_product(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::product_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Units
// ============================================================================
async fn list_units(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"units": controller::erp::units(&s.pool)?})))
}
async fn get_unit(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "unit", &id, |conn| {
        models::erp::Unit::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_unit(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewUnit>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let u = controller::erp::unit_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"unit": u}))))
}
async fn update_unit(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewUnit>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"unit": controller::erp::unit_update(&s.pool, &id, v)?})))
}
async fn delete_unit(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::unit_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Warehouses
// ============================================================================
async fn list_warehouses(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"warehouses": controller::erp::warehouses(&s.pool)?})))
}
async fn get_warehouse(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "warehouse", &id, |conn| {
        models::erp::Warehouse::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_warehouse(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewWarehouse>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let w = controller::erp::warehouse_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"warehouse": w}))))
}
async fn update_warehouse(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewWarehouse>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"warehouse": controller::erp::warehouse_update(&s.pool, &id, v)?})))
}
async fn delete_warehouse(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::warehouse_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Suppliers
// ============================================================================
async fn list_suppliers(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"suppliers": controller::erp::suppliers(&s.pool)?})))
}
async fn get_supplier(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "supplier", &id, |conn| {
        models::erp::Supplier::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_supplier(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewSupplier>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let s2 = controller::erp::supplier_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"supplier": s2}))))
}
async fn update_supplier(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewSupplier>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"supplier": controller::erp::supplier_update(&s.pool, &id, v)?})))
}
async fn delete_supplier(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::supplier_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Procurement
// ============================================================================
async fn list_procurement(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"requests": controller::erp::procurement(&s.pool)?})))
}
async fn get_procurement(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "request", &id, |conn| {
        models::erp::ProcurementRequest::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_procurement(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewProcurement>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let r = controller::erp::procurement_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"request": r}))))
}
async fn update_procurement(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewProcurement>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"request": controller::erp::procurement_update(&s.pool, &id, v)?})))
}
async fn delete_procurement(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::procurement_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_procurement_lines(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"lines": controller::erp::procurement_lines(&s.pool, &id)?})))
}
async fn create_procurement_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(mut v): Json<models::erp::NewProcurementLine>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    v.procurement_id = id.clone();
    let l = controller::erp::procurement_line_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"line": l}))))
}

// ============================================================================
// Purchase Orders
// ============================================================================
async fn list_pos(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"orders": controller::erp::purchase_orders(&s.pool)?})))
}
async fn get_po(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "order", &id, |conn| {
        models::erp::PurchaseOrder::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_po(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewPurchaseOrder>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let o = controller::erp::po_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"order": o}))))
}
async fn update_po(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewPurchaseOrder>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"order": controller::erp::po_update(&s.pool, &id, v)?})))
}
async fn delete_po(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::po_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_po_lines(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"lines": controller::erp::po_lines(&s.pool, &id)?})))
}
async fn get_po_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path((id, line_id)): Path<(String, String)>) -> ApiResult<Json<Value>> {
    one(&s, &c, "line", &line_id, |conn| {
        models::erp::PurchaseOrderLine::find_by_id(&line_id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_po_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(mut v): Json<models::erp::NewPurchaseOrderLine>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    v.purchase_order_id = id.clone();
    let l = controller::erp::po_line_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"line": l}))))
}
async fn update_po_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path((id, line_id)): Path<(String, String)>, Json(v): Json<models::erp::NewPurchaseOrderLine>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"line": controller::erp::po_line_update(&s.pool, &id, &line_id, v)?})))
}
async fn delete_po_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path((id, line_id)): Path<(String, String)>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::po_line_delete(&s.pool, &id, &line_id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Goods Receipts
// ============================================================================
async fn list_receipts(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"receipts": controller::erp::receipts(&s.pool)?})))
}
async fn get_receipt(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "receipt", &id, |conn| {
        models::erp::GoodsReceipt::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_receipt(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(payload): Json<Value>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let header = payload.get("header").cloned().unwrap_or(json!({}));
    let lines = payload.get("lines").cloned().unwrap_or(json!([]));
    let v: models::erp::NewGoodsReceipt = serde_json::from_value(header).map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let ls: Vec<models::erp::NewGoodsReceiptLine> = serde_json::from_value(lines).map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r = controller::erp::receipt_create(&s.pool, v, ls)?;
    Ok((StatusCode::CREATED, Json(json!({"receipt": r}))))
}
async fn delete_receipt(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::receipt_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_receipt_lines(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"lines": controller::erp::receipt_lines(&s.pool, &id)?})))
}
async fn create_receipt_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(mut v): Json<models::erp::NewGoodsReceiptLine>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    v.receipt_id = id.clone();
    let l = controller::erp::receipt_line_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"line": l}))))
}

// ============================================================================
// Inventory
// ============================================================================
async fn list_inventory(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"transactions": controller::erp::inventory_txs(&s.pool)?})))
}
async fn get_transaction(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "transaction", &id, |conn| {
        models::erp::InventoryTransaction::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn stock(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    let s2 = controller::erp::inventory_stock(&s.pool)?;
    Ok(Json(json!({"stock": s2})))
}

// ============================================================================
// Sales Orders
// ============================================================================
async fn list_sales(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"orders": controller::erp::sales_orders(&s.pool)?})))
}
async fn get_sales(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "order", &id, |conn| {
        models::erp::SalesOrder::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_sales(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewSalesOrder>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let o = controller::erp::sales_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"order": o}))))
}
async fn update_sales(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewSalesOrder>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"order": controller::erp::sales_update(&s.pool, &id, v)?})))
}
async fn delete_sales(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::sales_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_sales_lines(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"lines": controller::erp::sales_lines(&s.pool, &id)?})))
}
async fn get_sales_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path((id, line_id)): Path<(String, String)>) -> ApiResult<Json<Value>> {
    one(&s, &c, "line", &line_id, |conn| {
        models::erp::SalesLine::find_by_id(&line_id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_sales_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(mut v): Json<models::erp::NewSalesLine>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    v.sales_order_id = id.clone();
    let l = controller::erp::sales_line_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"line": l}))))
}
async fn update_sales_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path((id, line_id)): Path<(String, String)>, Json(v): Json<models::erp::NewSalesLine>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"line": controller::erp::sales_line_update(&s.pool, &id, &line_id, v)?})))
}
async fn delete_sales_line(State(s): State<AppState>, Extension(c): Extension<Claims>, Path((id, line_id)): Path<(String, String)>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::sales_line_delete(&s.pool, &id, &line_id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Shipments
// ============================================================================
async fn list_shipments(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"shipments": controller::erp::shipments(&s.pool)?})))
}
async fn get_shipment(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "shipment", &id, |conn| {
        models::erp::Shipment::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_shipment(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(payload): Json<Value>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let header = payload.get("header").cloned().unwrap_or(json!({}));
    let lines = payload.get("lines").cloned().unwrap_or(json!([]));
    let v: models::erp::NewShipment = serde_json::from_value(header).map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let ls: Vec<models::erp::NewSalesLine> = serde_json::from_value(lines).map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r = controller::erp::shipment_create(&s.pool, v, ls)?;
    Ok((StatusCode::CREATED, Json(json!({"shipment": r}))))
}
async fn update_shipment(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewShipment>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"shipment": controller::erp::shipment_update(&s.pool, &id, v)?})))
}
async fn delete_shipment(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::shipment_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Assets
// ============================================================================
async fn list_assets(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"assets": controller::erp::assets(&s.pool)?})))
}
async fn get_asset(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "asset", &id, |conn| {
        models::erp::Asset::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_asset(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewAsset>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let a = controller::erp::asset_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"asset": a}))))
}
async fn update_asset(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewAsset>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"asset": controller::erp::asset_update(&s.pool, &id, v)?})))
}
async fn delete_asset(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::asset_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Expenses
// ============================================================================
async fn list_expenses(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "read")?;
    Ok(Json(json!({"expenses": controller::erp::expenses(&s.pool)?})))
}
async fn get_expense(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    one(&s, &c, "expense", &id, |conn| {
        models::erp::Expense::find_by_id(&id).first(conn).map_err(|e| ApiError::new(StatusCode::NOT_FOUND, e.to_string()))
    })
}
async fn create_expense(State(s): State<AppState>, Extension(c): Extension<Claims>, Json(v): Json<models::erp::NewExpense>) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "commerce", "write")?;
    let e = controller::erp::expense_create(&s.pool, v)?;
    Ok((StatusCode::CREATED, Json(json!({"expense": e}))))
}
async fn update_expense(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>, Json(v): Json<models::erp::NewExpense>) -> ApiResult<Json<Value>> {
    need(&c, "commerce", "write")?;
    Ok(Json(json!({"expense": controller::erp::expense_update(&s.pool, &id, v)?})))
}
async fn delete_expense(State(s): State<AppState>, Extension(c): Extension<Claims>, Path(id): Path<String>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    controller::erp::expense_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn clear_expenses(State(s): State<AppState>, Extension(c): Extension<Claims>) -> ApiResult<StatusCode> {
    need(&c, "commerce", "write")?;
    for e in controller::erp::expenses(&s.pool)? {
        controller::erp::expense_delete(&s.pool, &e.id)?;
    }
    Ok(StatusCode::NO_CONTENT)
}