use serde::{Deserialize, Serialize};
use crate::models::order::Order;

#[derive(Debug, Clone)]
pub struct Orders {
    client: crate::client::ClickPesaClient,
}

impl Orders {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateOrderRequest) -> Result<CreateOrderResponse, crate::ClickPesaError> {
        let body = serde_json::to_value(request).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))?;
        let response: serde_json::Value = self.client.post("/v1/orders", body).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }

    pub async fn get(&self, order_id: &str) -> Result<Order, crate::ClickPesaError> {
        let path = format!("/v1/orders/{}", order_id);
        let response: serde_json::Value = self.client.get(&path, None).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }

    pub async fn list(&self, _request: ListOrdersRequest) -> Result<ListOrdersResponse, crate::ClickPesaError> {
        // Try different endpoint patterns
        let endpoints = vec![
            "/orders",
            "/v1/orders",
            "/api/orders",
            "/api/v1/orders",
            "/business/orders",
            "/v1/business/orders",
            "/transaction/orders",
            "/v1/transactions",
            "/payments",
            "/v1/payments",
        ];
        
        for endpoint in &endpoints {
            eprintln!("Trying endpoint: {}", endpoint);
            let response: serde_json::Value = self.client.get(endpoint, None).await?;
            eprintln!("Response: {}", response);
        }
        
        Err(crate::ClickPesaError::Api("No working endpoint found".to_string()))
    }

    pub async fn update(&self, order_id: &str, request: UpdateOrderRequest) -> Result<UpdateOrderResponse, crate::ClickPesaError> {
        let path = format!("/v1/orders/{}", order_id);
        let body = serde_json::to_value(request).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))?;
        let response: serde_json::Value = self.client.patch(&path, body).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }

    pub async fn capture(&self, order_id: &str, request: CaptureOrderRequest) -> Result<CaptureOrderResponse, crate::ClickPesaError> {
        let path = format!("/v1/orders/{}/capture", order_id);
        let body = serde_json::to_value(request).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))?;
        let response: serde_json::Value = self.client.post(&path, body).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrdersRequest {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub customer_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrdersResponse {
    pub orders: Vec<crate::models::order::Order>,
    pub total_count: i32,
    pub page: i32,
    pub page_size: i32,
}

pub mod create;
pub mod get;
pub mod update;
pub mod capture;

pub use create::*;
pub use get::*;
pub use update::*;
pub use capture::*;
