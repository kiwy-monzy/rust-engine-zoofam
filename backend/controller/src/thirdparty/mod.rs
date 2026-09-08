mod clickpesa;
mod beem;
mod notifty;

pub use clickpesa::ClickpesaProvider;
pub use beem::BeemAfricaProvider;
pub use notifty::NotiftyProvider;

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub currency: String,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    pub bic: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub id: String,
    pub status: String,
    pub channel: Option<String>,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub message: Option<String>,
    pub order_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResponse {
    pub id: String,
    pub status: String,
    pub channel: Option<String>,
    pub amount: String,
    pub currency: String,
    pub fee: Option<String>,
    pub order_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutLinkResponse {
    pub checkout_link: String,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillPayResponse {
    pub bill_pay_number: String,
    pub bill_reference: Option<String>,
    pub bill_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LipaNambaProvider {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub source: String,
    pub target: String,
    pub rate: f64,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub order_reference: String,
    pub status: String,
    pub channel: Option<String>,
    pub collected_amount: Option<String>,
    pub collected_currency: Option<String>,
    pub id: Option<String>,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    fn name(&self) -> &str;
    
    async fn get_balance(&self) -> Result<Vec<Balance>, String>;
    
    async fn preview_ussd_push(
        &self,
        amount: &str,
        order_id: &str,
        phone: Option<&str>,
        currency: &str,
    ) -> Result<serde_json::Value, String>;
    
    async fn initiate_ussd_push(
        &self,
        amount: &str,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<PaymentResponse, String>;
    
    async fn get_payment_status(&self, order_reference: &str) -> Result<Vec<PaymentResponse>, String>;
    
    async fn list_payments(&self, filters: Vec<(String, String)>) -> Result<serde_json::Value, String>;
    
    async fn preview_mobile_money_payout(
        &self,
        amount: f64,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, String>;
    
    async fn create_mobile_money_payout(
        &self,
        amount: f64,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<PayoutResponse, String>;
    
    async fn preview_bank_payout(
        &self,
        amount: f64,
        account_number: &str,
        bic: &str,
        order_id: &str,
        transfer_type: &str,
        currency: &str,
    ) -> Result<serde_json::Value, String>;
    
    async fn create_bank_payout(
        &self,
        amount: f64,
        account_number: &str,
        account_name: &str,
        bic: &str,
        order_id: &str,
        transfer_type: &str,
        currency: &str,
    ) -> Result<PayoutResponse, String>;
    
    async fn get_banks(&self) -> Result<Vec<Bank>, String>;
    
    async fn list_lipa_namba_providers(&self) -> Result<Vec<LipaNambaProvider>, String>;
    
    async fn create_billpay_control_number(
        &self,
        bill_reference: Option<&str>,
        amount: Option<f64>,
        description: Option<&str>,
        payment_mode: Option<&str>,
    ) -> Result<BillPayResponse, String>;
    
    async fn get_billpay_details(&self, bill_pay_number: &str) -> Result<serde_json::Value, String>;
    
    async fn generate_checkout_link(&self, request: CheckoutLinkRequest) -> Result<CheckoutLinkResponse, String>;
    
    async fn get_exchange_rates(&self, source: Option<&str>, target: Option<&str>) -> Result<Vec<ExchangeRate>, String>;
    
    fn verify_webhook(&self, _payload: &[u8], _signature: &str) -> Result<bool, String> {
        Ok(true)
    }
}

#[async_trait]
pub trait NotificationProvider: Send + Sync {
    fn name(&self) -> &str;
    
    async fn send_sms(&self, to: &str, message: &str) -> Result<String, String>;
    
    async fn send_bulk_sms(&self, recipients: Vec<&str>, message: &str) -> Result<Vec<String>, String>;
}

#[async_trait]
pub trait USSDProvider: Send + Sync {
    fn name(&self) -> &str;
    
    async fn initiate_ussd(
        &self,
        phone: &str,
        message: &str,
        callback_url: Option<&str>,
    ) -> Result<String, String>;
}

pub struct ThirdPartyServices {
    pub payment: Option<Arc<dyn PaymentProvider>>,
    pub notification: Option<Arc<dyn NotificationProvider>>,
    pub ussd: Option<Arc<dyn USSDProvider>>,
}

impl ThirdPartyServices {
    pub fn new() -> Self {
        Self {
            payment: None,
            notification: None,
            ussd: None,
        }
    }
    
    pub fn with_clickpesa(mut self) -> Result<Self, String> {
        self.payment = Some(Arc::new(ClickpesaProvider::new()?));
        Ok(self)
    }
    
    pub fn with_beem(api_key: String, sender_id: String) -> Self {
        let provider = BeemAfricaProvider::new(api_key, sender_id);
        Self {
            payment: None,
            notification: Some(Arc::new(provider.clone())),
            ussd: Some(Arc::new(provider)),
        }
    }
    
    pub fn with_notifty(api_key: String, sender_id: String) -> Self {
        let provider = NotiftyProvider::new(api_key, sender_id);
        Self {
            payment: None,
            notification: Some(Arc::new(provider.clone())),
            ussd: Some(Arc::new(provider)),
        }
    }
}

impl Default for ThirdPartyServices {
    fn default() -> Self {
        Self::new()
    }
}
