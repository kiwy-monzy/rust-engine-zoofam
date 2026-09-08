use serde::{Deserialize, Serialize};
use super::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
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
    pub links: Vec<OrderLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLink {
    pub rel: String,
    pub href: String,
    pub method: String,
}
