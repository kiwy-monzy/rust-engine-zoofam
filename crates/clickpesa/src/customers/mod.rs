use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Customers {
    client: crate::client::ClickPesaClient,
}

impl Customers {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateCustomerRequest) -> Result<Customer, crate::ClickPesaError> {
        let body = serde_json::to_value(request).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))?;
        let response: serde_json::Value = self.client.post("/v1/customer/customers", body).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }

    pub async fn get(&self, customer_id: &str) -> Result<Customer, crate::ClickPesaError> {
        let path = format!("/v1/customer/customers/{}", customer_id);
        let response: serde_json::Value = self.client.get(&path, None).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<Address>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub address: Option<Address>,
}
