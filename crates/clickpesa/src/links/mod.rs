#[derive(Debug, Clone)]
pub struct Links {
    client: crate::client::ClickPesaClient,
}

impl Links {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn generate_checkout(
        &self,
        amount: &str,
        order_reference: &str,
        currency: &str,
        description: Option<&str>,
        customer_name: Option<&str>,
        customer_email: Option<&str>,
        customer_phone: Option<&str>,
        return_url: Option<&str>,
        callback_url: Option<&str>,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut payload = serde_json::json!({
            "totalPrice": amount,
            "orderReference": order_reference,
            "orderCurrency": currency,
        });
        
        if let Some(desc) = description {
            payload["description"] = serde_json::json!(desc);
        }
        if let Some(name) = customer_name {
            payload["customerName"] = serde_json::json!(name);
        }
        if let Some(email) = customer_email {
            payload["customerEmail"] = serde_json::json!(email);
        }
        if let Some(phone) = customer_phone {
            payload["customerPhone"] = serde_json::json!(phone);
        }
        if let Some(ret) = return_url {
            payload["returnUrl"] = serde_json::json!(ret);
        }
        if let Some(cb) = callback_url {
            payload["callbackUrl"] = serde_json::json!(cb);
        }
        
        self.client.post("/third-parties/checkout-link/generate-checkout-url", payload).await
    }

    pub async fn generate_payout_link(
        &self,
        amount: &str,
        currency: &str,
        order_reference: &str,
        description: Option<&str>,
        beneficiary_name: Option<&str>,
        callback_url: Option<&str>,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut payload = serde_json::json!({
            "amount": amount,
            "currency": currency,
            "orderReference": order_reference,
        });
        
        if let Some(desc) = description {
            payload["description"] = serde_json::json!(desc);
        }
        if let Some(name) = beneficiary_name {
            payload["beneficiaryName"] = serde_json::json!(name);
        }
        if let Some(cb) = callback_url {
            payload["callbackUrl"] = serde_json::json!(cb);
        }
        
        self.client.post("/third-parties/payout-links/generate-payout-link", payload).await
    }
}