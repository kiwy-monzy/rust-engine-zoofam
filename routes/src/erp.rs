// ERP routes — was `routes::commerce`.
// All paths stay under `/commerce/*` for now to match the controller.
use axum::{extract::{Path, State}, http::StatusCode, Extension, Json, Router, routing::{delete, get, post, patch}};
use serde_json::{json, Value};
use crate::{middleware::{ApiError, ApiResult}, state::AppState};
use auth::Claims;

fn need(c:&Claims, m:&str, a:&str)->Result<(),ApiError>{ c.require(m,a).map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string())) }

pub fn erp_routes()->Router<AppState>{
    Router::new()
        .route("/commerce/products", get(list_products).post(create_product))
        .route("/commerce/products/:id", patch(update_product).delete(delete_product))
        .route("/commerce/warehouses", get(list_warehouses).post(create_warehouse))
        .route("/commerce/units", get(list_units).post(create_unit))
        .route("/commerce/suppliers", get(list_suppliers).post(create_supplier))
        .route("/commerce/procurement", get(list_procurement).post(create_procurement))
        .route("/commerce/purchase-orders", get(list_pos).post(create_po))
        .route("/commerce/purchase-orders/:id/lines", get(list_po_lines).post(create_po_line))
        .route("/commerce/goods-receipts", get(list_receipts).post(create_receipt))
        .route("/commerce/inventory/transactions", get(list_inventory))
        .route("/commerce/inventory/stock", get(stock))
        .route("/commerce/sales-orders", get(list_sales).post(create_sales))
        .route("/commerce/sales-orders/:id/lines", get(list_sales_lines).post(create_sales_line))
        .route("/commerce/shipments", post(create_shipment))
        .route("/commerce/assets", get(list_assets).post(create_asset))
        .route("/commerce/expenses", get(list_expenses).post(create_expense).delete(clear_expenses))
}

async fn list_products(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"products": controller::erp::products(&s.pool)?}))) }
async fn create_product(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewProduct>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"product": controller::erp::product_create(&s.pool, v)?})))) }
async fn update_product(State(s):State<AppState>, Extension(c):Extension<Claims>, Path(id):Path<String>, Json(v):Json<models::erp::UpdateProduct>)->ApiResult<Json<Value>>{ need(&c,"commerce","write")?; Ok(Json(json!({"product": controller::erp::product_update(&s.pool, &id, v)?}))) }
async fn delete_product(State(s):State<AppState>, Extension(c):Extension<Claims>, Path(id):Path<String>)->ApiResult<StatusCode>{ need(&c,"commerce","write")?; controller::erp::product_delete(&s.pool,&id)?; Ok(StatusCode::NO_CONTENT) }

async fn list_warehouses(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"warehouses": controller::erp::warehouses(&s.pool)?}))) }
async fn create_warehouse(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewWarehouse>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"warehouse": controller::erp::warehouse_create(&s.pool, v)?})))) }
async fn list_units(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"units": controller::erp::units(&s.pool)?}))) }
async fn create_unit(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewUnit>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"unit": controller::erp::unit_create(&s.pool,v)?})))) }
async fn list_suppliers(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"suppliers": controller::erp::suppliers(&s.pool)?}))) }
async fn create_supplier(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewSupplier>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"supplier": controller::erp::supplier_create(&s.pool,v)?})))) }

async fn list_procurement(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"requests": controller::erp::procurement(&s.pool)?}))) }
async fn create_procurement(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewProcurement>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"request": controller::erp::procurement_create(&s.pool,v)?})))) }

async fn list_pos(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"orders": controller::erp::purchase_orders(&s.pool)?}))) }
async fn create_po(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewPurchaseOrder>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"order": controller::erp::po_create(&s.pool,v)?})))) }
async fn list_po_lines(State(s):State<AppState>, Extension(c):Extension<Claims>, Path(id):Path<String>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"lines": controller::erp::po_lines(&s.pool,&id)?}))) }
async fn create_po_line(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewPurchaseOrderLine>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"line": controller::erp::po_line_create(&s.pool,v)?})))) }

async fn list_receipts(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"receipts": controller::erp::receipts(&s.pool)?}))) }
async fn create_receipt(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewGoodsReceipt>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"receipt": controller::erp::receipt_create(&s.pool,v, vec![])?})))) }

async fn list_inventory(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"transactions": controller::erp::inventory_txs(&s.pool)?}))) }
async fn stock(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; let s2=controller::erp::inventory_stock(&s.pool)?; Ok(Json(json!({"stock": s2})) )}

async fn list_sales(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"orders": controller::erp::sales_orders(&s.pool)?}))) }
async fn create_sales(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewSalesOrder>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"order": controller::erp::sales_create(&s.pool,v)?})))) }
async fn list_sales_lines(State(s):State<AppState>, Extension(c):Extension<Claims>, Path(id):Path<String>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"lines": controller::erp::sales_lines(&s.pool,&id)?}))) }
async fn create_sales_line(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewSalesLine>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"line": controller::erp::sales_line_create(&s.pool,v)?})))) }
async fn create_shipment(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewShipment>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"shipment": controller::erp::shipment_create(&s.pool,v, vec![])?})))) }

async fn list_assets(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"assets": controller::erp::assets(&s.pool)?}))) }
async fn create_asset(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewAsset>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"asset": controller::erp::asset_create(&s.pool,v)?})))) }

async fn list_expenses(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<Json<Value>>{ need(&c,"commerce","read")?; Ok(Json(json!({"expenses": controller::erp::expenses(&s.pool)?}))) }
async fn create_expense(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(v):Json<models::erp::NewExpense>)->ApiResult<(StatusCode, Json<Value>)>{ need(&c,"commerce","write")?; Ok((StatusCode::CREATED, Json(json!({"expense": controller::erp::expense_create(&s.pool,v)?})))) }
async fn clear_expenses(State(s):State<AppState>, Extension(c):Extension<Claims>)->ApiResult<StatusCode>{ need(&c,"commerce","write")?; for e in controller::erp::expenses(&s.pool)? { controller::erp::expense_delete(&s.pool,&e.id)?; } Ok(StatusCode::NO_CONTENT) }