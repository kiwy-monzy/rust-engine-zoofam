# Authorize Payment

Create a payment authorization.

```rust
let auth = clickpesa.payments().authorize(
    CreateAuthorizationRequest {
        order_id: "ORDER_ID".to_string(),
        amount: Money::from_f64(100.00, "KES"),
        currency: "KES".to_string(),
        capture_on: Some("2024-12-31T23:59:59Z".to_string()),
    }
).await?;
```
