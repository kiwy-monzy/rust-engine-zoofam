use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClickPesaError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Webhook verification failed: {0}")]
    WebhookVerification(String),
}

impl serde::Serialize for ClickPesaError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
