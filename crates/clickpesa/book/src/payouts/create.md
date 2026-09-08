# Create Payout

Create a payout to a recipient.

```rust
let payout = clickpesa.payouts().create(
    CreatePayoutRequest {
        amount: Money::from_f64(100.00, "KES"),
        currency: "KES".to_string(),
        recipient_type: Some("PHONE".to_string()),
        recipient_id: "+254712345678".to_string(),
        reference: Some("PAYOUT-001".to_string()),
        note: Some("Payment for services".to_string()),
        sender_batch_id: None,
    }
).await?;
```
