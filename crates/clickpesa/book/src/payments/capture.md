# Capture Payment

Capture an authorized payment.

```rust
let capture = clickpesa.payments().capture(
    CreateCaptureRequest {
        authorization_id: "AUTH_ID".to_string(),
        amount: Money::from_f64(100.00, "KES"),
        currency: "KES".to_string(),
        note_to_payer: Some("Payment received".to_string()),
        is_final_capture: Some(true),
    }
).await?;
```
