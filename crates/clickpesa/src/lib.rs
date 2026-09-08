pub mod client;
pub mod error;
pub mod auth;

#[cfg(feature = "orders")]
pub mod orders;
#[cfg(feature = "payments")]
pub mod payments;
#[cfg(feature = "subscriptions")]
pub mod subscriptions;
#[cfg(feature = "products")]
pub mod products;
#[cfg(feature = "customers")]
pub mod customers;
pub mod payouts;
#[cfg(feature = "invoices")]
pub mod invoices;
#[cfg(feature = "webhooks")]
pub mod webhooks;
pub mod account;
pub mod billpay;
pub mod exchange;
pub mod links;

pub mod models;

pub use client::ClickPesaClient;
pub use error::ClickPesaError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ClickPesa {
    client: ClickPesaClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickPesaConfig {
    pub api_key: String,
    pub secret_key: String,
    pub sandbox: bool,
    pub base_url: Option<String>,
}

impl ClickPesa {
    pub fn new(api_key: String, client_id: String, checksum_key: Option<String>, sandbox: bool) -> Self {
        let base_url = if sandbox {
            "https://api-sandbox.clickpesa.com".to_string()
        } else {
            "https://api.clickpesa.com".to_string()
        };
        
        Self {
            client: ClickPesaClient::new(api_key, client_id, checksum_key, base_url),
        }
    }

    pub fn from_env() -> Result<Self, ClickPesaError> {
        dotenvy::dotenv().ok();
        let api_key = std::env::var("CLICKPESA_API_KEY")
            .map_err(|_| ClickPesaError::Config("CLICKPESA_API_KEY not set".to_string()))?;
        let client_id = std::env::var("CLICKPESA_CLIENT_ID")
            .map_err(|_| ClickPesaError::Config("CLICKPESA_CLIENT_ID not set".to_string()))?;
        let checksum_key = std::env::var("CLICKPESA_CHECKSUM_KEY").ok();
        let sandbox = std::env::var("CLICKPESA_SANDBOX")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        
        Ok(Self::new(api_key, client_id, checksum_key, sandbox))
    }

    pub fn payments(&self) -> payments::Payments {
        payments::Payments::new(self.client.clone())
    }

    pub fn payouts(&self) -> payouts::Payouts {
        payouts::Payouts::new(self.client.clone())
    }

    pub fn account(&self) -> account::Account {
        account::Account::new(self.client.clone())
    }

    pub fn billpay(&self) -> billpay::BillPay {
        billpay::BillPay::new(self.client.clone())
    }

    pub fn exchange(&self) -> exchange::Exchange {
        exchange::Exchange::new(self.client.clone())
    }

    pub fn links(&self) -> links::Links {
        links::Links::new(self.client.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clickpesa_new() {
        let clickpesa = ClickPesa::new(
            "test_api_key".to_string(),
            "test_client_id".to_string(),
            Some("test_checksum".to_string()),
            true,
        );
        assert!(clickpesa.client.base_url.contains("sandbox"));
    }

    #[test]
    fn test_clickpesa_production() {
        let clickpesa = ClickPesa::new(
            "test_api_key".to_string(),
            "test_client_id".to_string(),
            None,
            false,
        );
        assert!(clickpesa.client.base_url.contains("api.clickpesa.com"));
    }
}