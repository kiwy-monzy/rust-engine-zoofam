use crate::middleware::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use controller::thirdparty::{CheckoutLinkRequest, ClickpesaProvider, PaymentProvider};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/thirdparty/clickpesa/balance", get(get_balance))
        .route("/thirdparty/clickpesa/banks", get(get_banks))
        .route("/thirdparty/clickpesa/exchange-rates", get(get_exchange_rates))
        .route("/thirdparty/clickpesa/payments/preview", post(preview_payment))
        .route("/thirdparty/clickpesa/payments/initiate", post(initiate_payment))
        .route("/thirdparty/clickpesa/payments/:order_ref/status", get(payment_status))
        .route("/thirdparty/clickpesa/payments", get(list_payments))
        .route("/thirdparty/clickpesa/payouts/mobile-money/preview", post(preview_mobile_payout))
        .route("/thirdparty/clickpesa/payouts/mobile-money/create", post(create_mobile_payout))
        .route("/thirdparty/clickpesa/payouts/bank/preview", post(preview_bank_payout))
        .route("/thirdparty/clickpesa/payouts/bank/create", post(create_bank_payout))
        .route("/thirdparty/clickpesa/billpay/create", post(create_billpay))
        .route("/thirdparty/clickpesa/billpay/:bill_pay_number", get(billpay_details))
        .route("/thirdparty/clickpesa/checkout-link", post(generate_checkout_link))
}

fn provider() -> Result<Arc<dyn PaymentProvider>, ApiError> {
    Ok(Arc::new(
        ClickpesaProvider::new()
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e))?,
    ) as Arc<dyn PaymentProvider>)
}

async fn get_balance() -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let balances = p.get_balance().await.map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "balance": balances })))
}

async fn get_banks() -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let banks = p.get_banks().await.map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "banks": banks })))
}

async fn get_exchange_rates(Query(params): Query<ExchangeQuery>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let rates = p
        .get_exchange_rates(params.source.as_deref(), params.target.as_deref())
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "rates": rates })))
}

#[derive(Deserialize)]
struct ExchangeQuery {
    source: Option<String>,
    target: Option<String>,
}

#[derive(Deserialize)]
struct PreviewPayment {
    amount: String,
    phone: Option<String>,
    currency: Option<String>,
}

async fn preview_payment(Json(payload): Json<PreviewPayment>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .preview_ussd_push(
            &payload.amount,
            "preview",
            payload.phone.as_deref(),
            payload.currency.as_deref().unwrap_or("TZS"),
        )
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct InitiatePayment {
    amount: String,
    phone: String,
    currency: String,
    order_reference: String,
}

async fn initiate_payment(Json(payload): Json<InitiatePayment>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .initiate_ussd_push(
            &payload.amount,
            &payload.phone,
            &payload.order_reference,
            &payload.currency,
        )
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({
        "id": result.id,
        "status": result.status,
        "order_reference": result.order_reference,
        "channel": result.channel,
        "amount": result.amount,
        "currency": result.currency,
    })))
}

async fn payment_status(Path(order_ref): Path<String>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .get_payment_status(&order_ref)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "payments": result })))
}

async fn list_payments(Query(params): Query<ListPaymentsQuery>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let filters: Vec<_> = params.into_iter().collect();
    let result = p
        .list_payments(filters)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct ListPaymentsQuery {
    status: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    skip: Option<i32>,
    limit: Option<i32>,
}

impl IntoIterator for ListPaymentsQuery {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut items = Vec::new();
        if let Some(s) = self.status {
            items.push(("status".to_string(), s));
        }
        if let Some(s) = self.start_date {
            items.push(("startDate".to_string(), s));
        }
        if let Some(s) = self.end_date {
            items.push(("endDate".to_string(), s));
        }
        if let Some(s) = self.skip {
            items.push(("skip".to_string(), s.to_string()));
        }
        if let Some(l) = self.limit {
            items.push(("limit".to_string(), l.to_string()));
        }
        items.into_iter()
    }
}

#[derive(Deserialize)]
struct MobilePayoutPreview {
    amount: f64,
    phone: String,
    currency: String,
}

async fn preview_mobile_payout(Json(payload): Json<MobilePayoutPreview>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let order_ref = format!("PAY-{}", uuid::Uuid::new_v4().to_string()[0..8].to_uppercase());
    let result = p
        .preview_mobile_money_payout(payload.amount, &payload.phone, &order_ref, &payload.currency)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct MobilePayoutCreate {
    amount: f64,
    phone: String,
    currency: String,
    order_reference: String,
}

async fn create_mobile_payout(Json(payload): Json<MobilePayoutCreate>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .create_mobile_money_payout(
            payload.amount,
            &payload.phone,
            &payload.order_reference,
            &payload.currency,
        )
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({
        "id": result.id,
        "status": result.status,
        "order_reference": result.order_reference,
        "channel": result.channel,
        "amount": result.amount,
        "currency": result.currency,
        "fee": result.fee,
    })))
}

#[derive(Deserialize)]
struct BankPayoutPreview {
    amount: f64,
    account_number: String,
    bic: String,
    transfer_type: String,
    currency: String,
}

async fn preview_bank_payout(Json(payload): Json<BankPayoutPreview>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let order_ref = format!("BANK-{}", uuid::Uuid::new_v4().to_string()[0..8].to_uppercase());
    let result = p
        .preview_bank_payout(
            payload.amount,
            &payload.account_number,
            &payload.bic,
            &order_ref,
            &payload.transfer_type,
            &payload.currency,
        )
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(result))
}

#[derive(Deserialize)]
struct BankPayoutCreate {
    amount: f64,
    account_number: String,
    account_name: String,
    bic: String,
    transfer_type: String,
    currency: String,
    order_reference: String,
}

async fn create_bank_payout(Json(payload): Json<BankPayoutCreate>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .create_bank_payout(
            payload.amount,
            &payload.account_number,
            &payload.account_name,
            &payload.bic,
            &payload.order_reference,
            &payload.transfer_type,
            &payload.currency,
        )
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({
        "id": result.id,
        "status": result.status,
        "order_reference": result.order_reference,
        "channel": result.channel,
        "amount": result.amount,
        "currency": result.currency,
        "fee": result.fee,
    })))
}

#[derive(Deserialize)]
struct BillPayCreate {
    bill_reference: Option<String>,
    amount: Option<f64>,
    description: Option<String>,
    payment_mode: Option<String>,
}

async fn create_billpay(Json(payload): Json<BillPayCreate>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .create_billpay_control_number(
            payload.bill_reference.as_deref(),
            payload.amount,
            payload.description.as_deref(),
            payload.payment_mode.as_deref(),
        )
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({
        "bill_pay_number": result.bill_pay_number,
        "bill_reference": result.bill_reference,
        "bill_amount": result.bill_amount,
    })))
}

async fn billpay_details(Path(bill_pay_number): Path<String>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .get_billpay_details(&bill_pay_number)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(result))
}

async fn generate_checkout_link(Json(payload): Json<CheckoutLinkRequest>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let result = p
        .generate_checkout_link(payload)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({
        "checkout_link": result.checkout_link,
        "client_id": result.client_id,
    })))
}

async fn webhook(Json(payload): Json<serde_json::Value>) -> Result<Json<Value>, ApiError> {
    tracing::info!("Clickpesa webhook received: {:?}", payload);
    Ok(Json(json!({ "received": true })))
}
