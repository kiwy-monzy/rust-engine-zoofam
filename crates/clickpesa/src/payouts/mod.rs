#[derive(Debug, Clone)]
pub struct Payouts {
    client: crate::client::ClickPesaClient,
}

impl Payouts {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn preview_mobile_money(
        &self,
        amount: f64,
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
        
        self.client.post("/third-parties/payouts/preview-mobile-money-payout", payload).await
    }

    pub async fn create_mobile_money(
        &self,
        amount: f64,
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
        
        self.client.post("/third-parties/payouts/create-mobile-money-payout", payload).await
    }

    pub async fn preview_bank(
        &self,
        amount: f64,
        account_number: &str,
        bic: &str,
        order_id: &str,
        transfer_type: &str,
        currency: &str,
        account_currency: &str,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let payload = serde_json::json!({
            "amount": amount,
            "accountNumber": account_number,
            "bic": bic,
            "orderReference": order_id,
            "transferType": transfer_type,
            "currency": currency,
            "accountCurrency": account_currency,
        });
        
        self.client.post("/third-parties/payouts/preview-bank-payout", payload).await
    }

    pub async fn create_bank(
        &self,
        amount: f64,
        account_number: &str,
        account_name: &str,
        bic: &str,
        order_id: &str,
        transfer_type: &str,
        currency: &str,
        account_currency: &str,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let payload = serde_json::json!({
            "amount": amount,
            "accountNumber": account_number,
            "accountName": account_name,
            "bic": bic,
            "orderReference": order_id,
            "transferType": transfer_type,
            "currency": currency,
            "accountCurrency": account_currency,
        });
        
        self.client.post("/third-parties/payouts/create-bank-payout", payload).await
    }

    pub async fn get_banks(&self) -> Result<serde_json::Value, crate::ClickPesaError> {
        self.client.get("/third-parties/list/banks", None).await
    }

    pub async fn get_status(&self, order_reference: &str) -> Result<serde_json::Value, crate::ClickPesaError> {
        self.client.get(&format!("/third-parties/payouts/{}", order_reference), None).await
    }

    pub async fn list_all(&self, filters: Vec<(String, String)>) -> Result<serde_json::Value, crate::ClickPesaError> {
        if filters.is_empty() {
            self.client.get("/third-parties/payouts/all", None).await
        } else {
            self.client.get("/third-parties/payouts/all", Some(filters)).await
        }
    }

    // Lipa Namba Payout APIs
    pub async fn list_lipa_namba_providers(&self) -> Result<serde_json::Value, crate::ClickPesaError> {
        self.client.get("/third-parties/payouts/list-lipa-namba-providers", None).await
    }

    pub async fn preview_lipa_namba(
        &self,
        amount: f64,
        lipa_namba: &str,
        provider_code: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let payload = serde_json::json!({
            "amount": amount,
            "lipaNamba": lipa_namba,
            "providerCode": provider_code,
            "orderReference": order_id,
            "currency": currency,
        });
        
        self.client.post("/third-parties/payouts/preview-lipa-namba-payout", payload).await
    }

    pub async fn create_lipa_namba(
        &self,
        amount: f64,
        lipa_namba: &str,
        provider_code: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let payload = serde_json::json!({
            "amount": amount,
            "lipaNamba": lipa_namba,
            "providerCode": provider_code,
            "orderReference": order_id,
            "currency": currency,
        });
        
        self.client.post("/third-parties/payouts/create-lipa-namba-payout", payload).await
    }
}