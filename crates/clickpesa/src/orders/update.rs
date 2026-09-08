use serde::{Deserialize, Serialize};
use crate::models::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderRequest {
    pub status: Option<String>,
    pub amount: Option<Money>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderResponse {
    pub id: String,
    pub status: String,
    pub updated_at: String,
}
