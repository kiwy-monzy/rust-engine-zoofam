#[derive(Debug, Clone)]
pub struct Exchange {
    client: crate::client::ClickPesaClient,
}

impl Exchange {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn get_rates(&self, source_currency: Option<&str>, target_currency: Option<&str>) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut params = Vec::new();
        if let Some(src) = source_currency {
            params.push(("source".to_string(), src.to_string()));
        }
        if let Some(tgt) = target_currency {
            params.push(("target".to_string(), tgt.to_string()));
        }
        
        if params.is_empty() {
            self.client.get("/third-parties/exchange-rates/all", None).await
        } else {
            self.client.get("/third-parties/exchange-rates/all", Some(params)).await
        }
    }
}