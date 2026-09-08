use serde::{Deserialize, Serialize};
use super::Money;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub status: String,
    pub amount: Money,
    pub currency: String,
    pub payment_method: String,
    pub customer_id: String,
    pub order_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSource {
    pub payment_type: Option<String>,
    pub vault_id: Option<String>,
}
