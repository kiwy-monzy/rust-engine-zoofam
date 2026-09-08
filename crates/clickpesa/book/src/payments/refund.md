# Refund Payment

Refund a captured payment.

```rust
let refund = clickpesa.payments().refund(
    CreateRefundRequest {
        capture_id: "CAPTURE_ID".to_string(),
        amount: Money::from_f64(50.00, "KES"),
        currency: "KES".to_string(),
        reason: Some("Customer request".to_string()),
    }
).await?;
```
