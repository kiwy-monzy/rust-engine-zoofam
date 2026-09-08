use serde::{Deserialize, Serialize};
use crate::models::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureOrderRequest {
    pub amount: Option<Money>,
    pub note_to_payer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureOrderResponse {
    pub id: String,
    pub status: String,
    pub amount: Money,
    pub currency: String,
    pub captured_at: String,
    pub reference: Option<String>,
}
