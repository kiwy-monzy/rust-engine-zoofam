use serde::{Deserialize, Serialize};
use crate::models::order::Order;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrderResponse {
    pub order: Order,
}
