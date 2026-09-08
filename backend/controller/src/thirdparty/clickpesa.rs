use async_trait::async_trait;
use clickpesa::ClickPesa;
use serde::Deserialize;

use super::{
    Balance, Bank, BillPayResponse, CheckoutLinkRequest, CheckoutLinkResponse,
    ExchangeRate, LipaNambaProvider, PaymentProvider, PaymentResponse, PayoutResponse,
};

pub struct ClickpesaProvider {
    client: ClickPesa,
}

impl ClickpesaProvider {
    pub fn new() -> Result<Self, String> {
        let client = ClickPesa::from_env().map_err(|e| e.to_string())?;
        Ok(Self { client })
    }
    
    pub fn with_config(api_key: String, client_id: String, checksum_key: Option<String>, sandbox: bool) -> Self {
        let client = ClickPesa::new(api_key, client_id, checksum_key, sandbox);
        Self { client }
    }
}

impl Default for ClickpesaProvider {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Clickpesa provider")
    }
}

#[derive(Debug, Deserialize)]
struct ClickpesaBalance {
    #[serde(rename = "currency")]
    currency: String,
    #[serde(rename = "balance")]
    balance: f64,
}

#[derive(Debug, Deserialize)]
struct ClickpesaBalanceResponse {
    #[serde(rename = "balances")]
    balances: Vec<ClickpesaBalance>,
}

#[derive(Debug, Deserialize)]
struct ClickpesaBank {
    #[serde(rename = "bic")]
    bic: String,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct ClickpesaPayment {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "channel")]
    channel: Option<String>,
    #[serde(rename = "collectedAmount")]
    collected_amount: Option<String>,
    #[serde(rename = "collectedCurrency")]
    collected_currency: Option<String>,
    #[serde(rename = "message")]
    message: Option<String>,
    #[serde(rename = "orderReference")]
    order_reference: String,
}

#[derive(Debug, Deserialize)]
struct ClickpesaBillPay {
    #[serde(rename = "billPayNumber")]
    bill_pay_number: String,
}

#[derive(Debug, Deserialize)]
struct ClickpesaCheckoutLink {
    #[serde(rename = "checkoutLink")]
    checkout_link: String,
    #[serde(rename = "clientId")]
    client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClickpesaExchangeRate {
    #[serde(rename = "source")]
    source: String,
    #[serde(rename = "target")]
    target: String,
    #[serde(rename = "rate")]
    rate: f64,
    #[serde(rename = "date")]
    date: String,
}

#[derive(Debug, Deserialize)]
struct ClickpesaProviderInfo {
    #[serde(rename = "code")]
    code: String,
    #[serde(rename = "name")]
    name: String,
}

#[derive(Debug, Deserialize)]
struct ClickpesaPayout {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "channel")]
    channel: Option<String>,
    #[serde(rename = "amount")]
    amount: String,
    #[serde(rename = "currency")]
    currency: String,
    #[serde(rename = "fee")]
    fee: Option<String>,
    #[serde(rename = "orderReference")]
    order_reference: String,
}

#[async_trait]
impl PaymentProvider for ClickpesaProvider {
    fn name(&self) -> &str {
        "clickpesa"
    }
    
    async fn get_balance(&self) -> Result<Vec<Balance>, String> {
        let response = self.client.account().get_balance().await.map_err(|e| e.to_string())?;
        let data: ClickpesaBalanceResponse = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(data.balances.into_iter().map(|b| Balance {
            currency: b.currency,
            balance: b.balance,
        }).collect())
    }
    
    async fn preview_ussd_push(
        &self,
        amount: &str,
        order_id: &str,
        phone: Option<&str>,
        currency: &str,
    ) -> Result<serde_json::Value, String> {
        self.client.payments()
            .preview_ussd_push(amount, order_id, phone, currency, false)
            .await.map_err(|e| e.to_string())
    }
    
    async fn initiate_ussd_push(
        &self,
        amount: &str,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<PaymentResponse, String> {
        let response = self.client.payments()
            .initiate_ussd_push(amount, phone, order_id, currency)
            .await.map_err(|e| e.to_string())?;
        let data: ClickpesaPayment = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(PaymentResponse {
            id: data.id,
            status: data.status,
            channel: data.channel,
            amount: data.collected_amount,
            currency: data.collected_currency,
            message: data.message,
            order_reference: data.order_reference,
        })
    }
    
    async fn get_payment_status(&self, order_reference: &str) -> Result<Vec<PaymentResponse>, String> {
        let response = self.client.payments().get_status(order_reference).await.map_err(|e| e.to_string())?;
        let data: Vec<ClickpesaPayment> = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(data.into_iter().map(|p| PaymentResponse {
            id: p.id,
            status: p.status,
            channel: p.channel,
            amount: p.collected_amount,
            currency: p.collected_currency,
            message: p.message,
            order_reference: p.order_reference,
        }).collect())
    }
    
    async fn list_payments(&self, filters: Vec<(String, String)>) -> Result<serde_json::Value, String> {
        self.client.payments().list_all(filters).await.map_err(|e| e.to_string())
    }
    
    async fn preview_mobile_money_payout(
        &self,
        amount: f64,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<serde_json::Value, String> {
        self.client.payouts()
            .preview_mobile_money(amount, phone, order_id, currency)
            .await.map_err(|e| e.to_string())
    }
    
    async fn create_mobile_money_payout(
        &self,
        amount: f64,
        phone: &str,
        order_id: &str,
        currency: &str,
    ) -> Result<PayoutResponse, String> {
        let response = self.client.payouts()
            .create_mobile_money(amount, phone, order_id, currency)
            .await.map_err(|e| e.to_string())?;
        let data: ClickpesaPayout = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(PayoutResponse {
            id: data.id,
            status: data.status,
            channel: data.channel,
            amount: data.amount,
            currency: data.currency,
            fee: data.fee,
            order_reference: data.order_reference,
        })
    }
    
    async fn preview_bank_payout(
        &self,
        amount: f64,
        account_number: &str,
        bic: &str,
        order_id: &str,
        transfer_type: &str,
        currency: &str,
    ) -> Result<serde_json::Value, String> {
        self.client.payouts()
            .preview_bank(amount, account_number, bic, order_id, transfer_type, currency, "TZS")
            .await.map_err(|e| e.to_string())
    }
    
    async fn create_bank_payout(
        &self,
        amount: f64,
        account_number: &str,
        account_name: &str,
        bic: &str,
        order_id: &str,
        transfer_type: &str,
        currency: &str,
    ) -> Result<PayoutResponse, String> {
        let response = self.client.payouts()
            .create_bank(amount, account_number, account_name, bic, order_id, transfer_type, currency, "TZS")
            .await.map_err(|e| e.to_string())?;
        let data: ClickpesaPayout = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(PayoutResponse {
            id: data.id,
            status: data.status,
            channel: data.channel,
            amount: data.amount,
            currency: data.currency,
            fee: data.fee,
            order_reference: data.order_reference,
        })
    }
    
    async fn get_banks(&self) -> Result<Vec<Bank>, String> {
        let response = self.client.payouts().get_banks().await.map_err(|e| e.to_string())?;
        let data: Vec<ClickpesaBank> = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(data.into_iter().map(|b| Bank {
            bic: b.bic,
            name: b.name,
            value: b.value,
        }).collect())
    }
    
    async fn list_lipa_namba_providers(&self) -> Result<Vec<LipaNambaProvider>, String> {
        let response = self.client.payouts().list_lipa_namba_providers().await.map_err(|e| e.to_string())?;
        let data: Vec<ClickpesaProviderInfo> = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(data.into_iter().map(|p| LipaNambaProvider {
            code: p.code,
            name: p.name,
        }).collect())
    }
    
    async fn create_billpay_control_number(
        &self,
        bill_reference: Option<&str>,
        amount: Option<f64>,
        description: Option<&str>,
        payment_mode: Option<&str>,
    ) -> Result<BillPayResponse, String> {
        let response = self.client.billpay()
            .create_order_control_number(bill_reference, amount, description, payment_mode)
            .await.map_err(|e| e.to_string())?;
        let data: ClickpesaBillPay = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(BillPayResponse {
            bill_pay_number: data.bill_pay_number,
            bill_reference: bill_reference.map(String::from),
            bill_amount: amount.map(|a| a.to_string()),
        })
    }
    
    async fn get_billpay_details(&self, bill_pay_number: &str) -> Result<serde_json::Value, String> {
        self.client.billpay().get_details(bill_pay_number).await.map_err(|e| e.to_string())
    }
    
    async fn generate_checkout_link(&self, request: CheckoutLinkRequest) -> Result<CheckoutLinkResponse, String> {
        let response = self.client.links().generate_checkout(
            &request.amount,
            &request.order_reference,
            &request.currency,
            request.description.as_deref(),
            request.customer_name.as_deref(),
            request.customer_email.as_deref(),
            request.customer_phone.as_deref(),
            request.return_url.as_deref(),
            request.callback_url.as_deref(),
        ).await.map_err(|e| e.to_string())?;
        let data: ClickpesaCheckoutLink = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(CheckoutLinkResponse {
            checkout_link: data.checkout_link,
            client_id: data.client_id,
        })
    }
    
    async fn get_exchange_rates(&self, source: Option<&str>, target: Option<&str>) -> Result<Vec<ExchangeRate>, String> {
        let response = self.client.exchange().get_rates(source, target).await.map_err(|e| e.to_string())?;
        let data: Vec<ClickpesaExchangeRate> = serde_json::from_value(response)
            .map_err(|e| format!("parse error: {}", e))?;
        Ok(data.into_iter().map(|r| ExchangeRate {
            source: r.source,
            target: r.target,
            rate: r.rate,
            date: r.date,
        }).collect())
    }
}
