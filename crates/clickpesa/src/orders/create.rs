use serde::{Deserialize, Serialize};
use crate::models::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub amount: Money,
    pub currency: String,
    pub description: Option<String>,
    pub customer_id: Option<String>,
    pub payment_method: Option<String>,
    pub reference: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub return_url: Option<String>,
    pub cancel_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub id: String,
    pub status: String,
    pub amount: Money,
    pub currency: String,
    pub description: Option<String>,
    pub customer_id: Option<String>,
    pub payment_method: Option<String>,
    pub reference: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub rel: String,
    pub href: String,
    pub method: String,
}
