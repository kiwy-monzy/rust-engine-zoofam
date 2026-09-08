use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Products {
    client: crate::client::ClickPesaClient,
}

impl Products {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateProductRequest) -> Result<Product, crate::ClickPesaError> {
        let body = serde_json::to_value(request).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))?;
        let response: serde_json::Value = self.client.post("/v1/catalogs/products", body).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }

    pub async fn list(&self, request: ListProductsRequest) -> Result<ListProductsResponse, crate::ClickPesaError> {
        let path = format!("/v1/catalogs/products?page={}&page_size={}", 
            request.page.unwrap_or(1), request.page_size.unwrap_or(20));
        let response: serde_json::Value = self.client.get(&path, None).await?;
        serde_json::from_value(response).map_err(|e| crate::ClickPesaError::Parse(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub type_: String,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub home_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub type_: String,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub home_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProductsRequest {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProductsResponse {
    pub products: Vec<Product>,
    pub total_count: i32,
    pub page: i32,
    pub page_size: i32,
}
