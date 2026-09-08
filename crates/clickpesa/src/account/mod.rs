#[derive(Debug, Clone)]
pub struct Account {
    client: crate::client::ClickPesaClient,
}

impl Account {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn get_balance(&self) -> Result<serde_json::Value, crate::ClickPesaError> {
        self.client.get("/third-parties/account/balance", None).await
    }

    pub async fn get_statement(&self, currency: &str, start_date: Option<&str>, end_date: Option<&str>) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut params = vec![("currency".to_string(), currency.to_string())];
        
        if let Some(start) = start_date {
            params.push(("startDate".to_string(), start.to_string()));
        }
        if let Some(end) = end_date {
            params.push(("endDate".to_string(), end.to_string()));
        }
        
        self.client.get("/third-parties/account/statement", Some(params)).await
    }
}