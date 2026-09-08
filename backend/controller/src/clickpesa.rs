use clickpesa::ClickPesa;

#[derive(Clone)]
pub struct ClickpesaService {
    client: ClickPesa,
}

impl ClickpesaService {
    pub fn new() -> Result<Self, String> {
        let client = ClickPesa::from_env().map_err(|e| e.to_string())?;
        Ok(Self { client })
    }

    pub fn with_config(api_key: String, client_id: String, checksum_key: Option<String>, sandbox: bool) -> Self {
        let client = ClickPesa::new(api_key, client_id, checksum_key, sandbox);
        Self { client }
    }

    pub async fn get_balance(&self) -> Result<serde_json::Value, String> {
        self.client.account().get_balance().await.map_err(|e| e.to_string())
    }

    pub async fn preview_ussd_push(
        &self,
        amount: &str,
        order_id: &str,
        phone: Option<&str>,
        currency: &str,
        fetch_sender: bool,
    ) -> Result<serde_json::Value, String> {
        self.client.payments().preview_ussd_push(amount, order_id, phone, currency, fetch_sender)
            .await.map_err(|e| e.to_string())
    }

    pub async fn initiate_ussd_push(
        &self,
        amount: &str,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, String> {
        self.client.payments().initiate_ussd_push(amount, phone, order_id, currency)
            .await.map_err(|e| e.to_string())
    }

    pub async fn get_payment_status(&self, order_reference: &str) -> Result<serde_json::Value, String> {
        self.client.payments().get_status(order_reference).await.map_err(|e| e.to_string())
    }

    pub async fn list_payments(&self, filters: Vec<(String, String)>) -> Result<serde_json::Value, String> {
        self.client.payments().list_all(filters).await.map_err(|e| e.to_string())
    }

    pub async fn preview_mobile_money_payout(
        &self,
        amount: f64,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, String> {
        self.client.payouts().preview_mobile_money(amount, phone, order_id, currency)
            .await.map_err(|e| e.to_string())
    }

    pub async fn create_mobile_money_payout(
        &self,
        amount: f64,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, String> {
        self.client.payouts().create_mobile_money(amount, phone, order_id, currency)
            .await.map_err(|e| e.to_string())
    }

    pub async fn get_payout_status(&self, order_reference: &str) -> Result<serde_json::Value, String> {
        self.client.payouts().get_status(order_reference).await.map_err(|e| e.to_string())
    }

    pub async fn list_payouts(&self, filters: Vec<(String, String)>) -> Result<serde_json::Value, String> {
        self.client.payouts().list_all(filters).await.map_err(|e| e.to_string())
    }

    pub async fn get_banks(&self) -> Result<serde_json::Value, String> {
        self.client.payouts().get_banks().await.map_err(|e| e.to_string())
    }

    pub async fn create_billpay_control_number(
        &self,
        bill_reference: Option<&str>,
        amount: Option<f64>,
        description: Option<&str>,
        payment_mode: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        self.client.billpay().create_order_control_number(bill_reference, amount, description, payment_mode)
            .await.map_err(|e| e.to_string())
    }

    pub async fn get_billpay_details(&self, bill_pay_number: &str) -> Result<serde_json::Value, String> {
        self.client.billpay().get_details(bill_pay_number).await.map_err(|e| e.to_string())
    }

    pub async fn generate_checkout_link(
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
    ) -> Result<serde_json::Value, String> {
        self.client.links().generate_checkout(
            amount,
            order_reference,
            currency,
            description,
            customer_name,
            customer_email,
            customer_phone,
            return_url,
            callback_url,
        ).await.map_err(|e| e.to_string())
    }

    pub async fn get_exchange_rates(&self, source: Option<&str>, target: Option<&str>) -> Result<serde_json::Value, String> {
        self.client.exchange().get_rates(source, target).await.map_err(|e| e.to_string())
    }
}

impl Default for ClickpesaService {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Clickpesa service")
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PaymentRequest {
    pub amount: String,
    pub phone: String,
    pub currency: String,
    pub order_reference: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PayoutRequest {
    pub amount: f64,
    pub phone: String,
    pub currency: String,
    pub order_reference: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckoutLinkRequest {
    pub amount: String,
    pub currency: String,
    pub order_reference: String,
    pub description: Option<String>,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub return_url: Option<String>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClickpesaWebhookPayload {
    #[serde(rename = "orderReference")]
    pub order_reference: String,
    pub status: String,
    pub channel: Option<String>,
    #[serde(rename = "collectedAmount")]
    pub collected_amount: Option<String>,
    #[serde(rename = "collectedCurrency")]
    pub collected_currency: Option<String>,
    pub id: Option<String>,
}

impl ClickpesaWebhookPayload {
    pub fn is_success(&self) -> bool {
        matches!(self.status.as_str(), "SUCCESS" | "SETTLED")
    }
}
