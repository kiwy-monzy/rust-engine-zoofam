use serde::{Deserialize, Serialize};
use crate::models::Money;

#[derive(Debug, Clone)]
pub struct Invoices {
    client: crate::client::ClickPesaClient,
}

impl Invoices {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateInvoiceRequest) -> Result<Invoice, crate::ClickPesaError> {
        let body = serde_json::to_value(request).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))?;
        let response: serde_json::Value = self.client.post("/v1/invoicing/invoices", body).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub number: String,
    pub status: String,
    pub customer_id: String,
    pub amount: Money,
    pub currency: String,
    pub due_date: String,
    pub issued_date: String,
    pub line_items: Vec<LineItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub quantity: String,
    pub unit_amount: Money,
    pub amount: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub number: String,
    pub customer_id: String,
    pub amount: Money,
    pub currency: String,
    pub due_date: String,
    pub issued_date: String,
    pub line_items: Vec<LineItemInput>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemInput {
    pub name: String,
    pub description: Option<String>,
    pub quantity: String,
    pub unit_amount: Money,
}
