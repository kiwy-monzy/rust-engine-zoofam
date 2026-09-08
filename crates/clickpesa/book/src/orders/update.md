# Update Order

Update an existing order.

```rust
let updated = clickpesa.orders().update(
    "ORDER_ID",
    UpdateOrderRequest {
        status: Some("COMPLETED".to_string()),
        amount: None,
        description: Some("Updated description".to_string()),
        metadata: None,
    }
).await?;
```
