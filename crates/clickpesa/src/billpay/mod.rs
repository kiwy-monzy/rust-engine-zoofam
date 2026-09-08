

#[derive(Debug, Clone)]
pub struct BillPay {
    client: crate::client::ClickPesaClient,
}

impl BillPay {
    pub fn new(client: crate::client::ClickPesaClient) -> Self {
        Self { client }
    }

    pub async fn create_order_control_number(
        &self,
        bill_reference: Option<&str>,
        amount: Option<f64>,
        description: Option<&str>,
        payment_mode: Option<&str>,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut payload = serde_json::json!({});
        
        if let Some(ref br) = bill_reference {
            payload["billReference"] = serde_json::json!(br);
        }
        if let Some(a) = amount {
            payload["billAmount"] = serde_json::json!(a);
        }
        if let Some(desc) = description {
            payload["billDescription"] = serde_json::json!(desc);
        }
        if let Some(pm) = payment_mode {
            payload["billPaymentMode"] = serde_json::json!(pm);
        }
        
        self.client.post("/third-parties/billpay/create-order-control-number", payload).await
    }

    pub async fn create_customer_control_number(
        &self,
        customer_name: &str,
        phone: Option<&str>,
        email: Option<&str>,
        bill_reference: Option<&str>,
        amount: Option<f64>,
        description: Option<&str>,
        payment_mode: Option<&str>,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut payload = serde_json::json!({
            "customerName": customer_name,
        });
        
        if let Some(p) = phone {
            payload["customerPhone"] = serde_json::json!(p);
        }
        if let Some(e) = email {
            payload["customerEmail"] = serde_json::json!(e);
        }
        if let Some(ref br) = bill_reference {
            payload["billReference"] = serde_json::json!(br);
        }
        if let Some(a) = amount {
            payload["billAmount"] = serde_json::json!(a);
        }
        if let Some(desc) = description {
            payload["billDescription"] = serde_json::json!(desc);
        }
        if let Some(pm) = payment_mode {
            payload["billPaymentMode"] = serde_json::json!(pm);
        }
        
        self.client.post("/third-parties/billpay/create-customer-control-number", payload).await
    }

    pub async fn get_details(&self, bill_pay_number: &str) -> Result<serde_json::Value, crate::ClickPesaError> {
        self.client.get(&format!("/third-parties/billpay/{}", bill_pay_number), None).await
    }

    pub async fn update_reference(
        &self,
        bill_pay_number: &str,
        amount: Option<f64>,
        description: Option<&str>,
        status: Option<&str>,
        payment_mode: Option<&str>,
    ) -> Result<serde_json::Value, crate::ClickPesaError> {
        let mut payload = serde_json::json!({});
        
        if let Some(a) = amount {
            payload["billAmount"] = serde_json::json!(a);
        }
        if let Some(desc) = description {
            payload["billDescription"] = serde_json::json!(desc);
        }
        if let Some(s) = status {
            payload["billStatus"] = serde_json::json!(s);
        }
        if let Some(pm) = payment_mode {
            payload["billPaymentMode"] = serde_json::json!(pm);
        }
        
        self.client.patch(&format!("/third-parties/billpay/{}", bill_pay_number), payload).await
    }
}