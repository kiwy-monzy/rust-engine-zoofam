use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub value: String,
    pub currency_code: String,
}

impl Money {
    pub fn new(value: impl Into<String>, currency_code: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            currency_code: currency_code.into(),
        }
    }

    pub fn from_f64(value: f64, currency_code: impl Into<String>) -> Self {
        Self {
            value: format!("{:.2}", value),
            currency_code: currency_code.into(),
        }
    }
}
