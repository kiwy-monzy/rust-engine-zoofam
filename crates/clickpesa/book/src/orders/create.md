# Create Order

Create a new order for payment.

```rust
use clickpesa::{ClickPesa, models::Money};

let order = clickpesa.orders().create(
    CreateOrderRequest {
        amount: Money::from_f64(100.00, "KES"),
        currency: "KES".to_string(),
        description: Some("Order #12345".to_string()),
        customer_id: Some("CUST123".to_string()),
        payment_method: None,
        reference: Some("REF-001".to_string()),
        metadata: None,
        return_url: Some("https://example.com/success".to_string()),
        cancel_url: Some("https://example.com/cancel".to_string()),
    }
).await?;
```
