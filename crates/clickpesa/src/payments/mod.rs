use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Payments {
    client: crate::client::ClickPesaClient,
}

impl Payments {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn preview_ussd_push(
        &self,
        amount: &str,
        order_id: &str,
        phone: Option<&str>,
        currency: &str,
        fetch_sender_details: bool,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut payload = serde_json::json!({
            "amount": amount,
            "currency": currency,
            "orderReference": order_id,
            "fetchSenderDetails": fetch_sender_details,
        });
        
        if let Some(p) = phone {
            payload["phoneNumber"] = serde_json::json!(p);
        }
        
        self.client.post("/third-parties/payments/preview-ussd-push-request", payload).await
    }

    pub async fn initiate_ussd_push(
        &self,
        amount: &str,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let payload = serde_json::json!({
            "amount": amount,
            "phoneNumber": phone,
            "currency": currency,
            "orderReference": order_id,
        });
        
        self.client.post("/third-parties/payments/initiate-ussd-push-request", payload).await
    }

    pub async fn preview_card(
        &self,
        amount: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let payload = serde_json::json!({
            "amount": amount,
            "currency": currency,
            "orderReference": order_id,
        });
        
        self.client.post("/third-parties/payments/preview-card-payment", payload).await
    }

    pub async fn initiate_card(
        &self,
        amount: &str,
        order_id: &str,
        customer: serde_json::Value,
        currency: &str,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let payload = serde_json::json!({
            "amount": amount,
            "orderReference": order_id,
            "currency": currency,
            "customer": customer,
        });
        
        self.client.post("/third-parties/payments/initiate-card-payment", payload).await
    }

    pub async fn get_status(&self, order_reference: &str) -> Result<serde_json::Value, crate::ClickPesaError> {
        self.client.get(&format!("/third-parties/payments/{}", order_reference), None).await
    }

    pub async fn list_all(&self, filters: Vec<(String, String)>) -> Result<serde_json::Value, crate::ClickPesaError> {
        if filters.is_empty() {
            self.client.get("/third-parties/payments/all", None).await
        } else {
            self.client.get("/third-parties/payments/all", Some(filters)).await
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdPushRequest {
    pub amount: String,
    pub phone_number: String,
    pub currency: String,
    pub order_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardPaymentRequest {
    pub amount: String,
    pub order_reference: String,
    pub currency: String,
    pub customer: CardCustomer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardCustomer {
    pub id: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
}