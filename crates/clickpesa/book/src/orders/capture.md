# Capture Order

Capture an authorized order payment.

```rust
let captured = clickpesa.orders().capture(
    "ORDER_ID",
    CaptureOrderRequest {
        amount: Some(Money::from_f64(100.00, "KES")),
        note_to_payer: Some("Payment captured".to_string()),
    }
).await?;
```
