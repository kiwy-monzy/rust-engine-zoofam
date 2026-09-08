use serde::{Deserialize, Serialize};
use crate::models::Money;

#[derive(Debug, Clone)]
pub struct Subscriptions {
    client: crate::client::ClickPesaClient,
}

impl Subscriptions {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateSubscriptionRequest) -> Result<Subscription, crate::ClickPesaError> {
        let body = serde_json::to_value(request).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))?;
        let response: serde_json::Value = self.client.post("/v1/billing/subscriptions", body).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }

    pub async fn get(&self, subscription_id: &str) -> Result<Subscription, crate::ClickPesaError> {
        let path = format!("/v1/billing/subscriptions/{}", subscription_id);
        let response: serde_json::Value = self.client.get(&path, None).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }

    pub async fn list_plans(&self, request: ListPlansRequest) -> Result<ListPlansResponse, crate::ClickPesaError> {
        let mut path = "/v1/billing/plans?page=1".to_string();
        if let Some(page) = request.page {
            path = format!("{}&page={}", path, page);
        }
        if let Some(page_size) = request.page_size {
            path = format!("{}&page_size={}", path, page_size);
        }
        let response: serde_json::Value = self.client.get(&path, None).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub plan_id: String,
    pub customer_id: String,
    pub start_time: Option<String>,
    pub quantity: Option<String>,
    pub shipping_amount: Option<Money>,
    pub application_context: Option<ApplicationContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationContext {
    pub brand_name: Option<String>,
    pub locale: Option<String>,
    pub shipping_preference: Option<String>,
    pub user_action: Option<String>,
    pub return_url: Option<String>,
    pub cancel_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub status: String,
    pub plan_id: String,
    pub customer_id: String,
    pub start_time: String,
    pub quantity: Option<String>,
    pub shipping_amount: Option<Money>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPlansRequest {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub product_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPlansResponse {
    pub plans: Vec<Plan>,
    pub total_count: i32,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub product_id: String,
    pub billing_cycles: Vec<BillingCycle>,
    pub payment_preferences: PaymentPreferences,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingCycle {
    pub frequency: Frequency,
    pub tenure_type: String,
    pub sequence: i32,
    pub total_cycles: i32,
    pub pricing_scheme: PricingScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frequency {
    pub interval_unit: String,
    pub interval_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingScheme {
    pub fixed_price: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentPreferences {
    pub auto_bill_outstanding: bool,
    pub setup_fee: Option<Money>,
    pub setup_fee_failure_action: String,
    pub payment_failure_threshold: i32,
}
