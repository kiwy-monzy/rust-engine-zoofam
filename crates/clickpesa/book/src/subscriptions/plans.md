# List Plans

List all available subscription plans.

```rust
let plans = clickpesa.subscriptions().list_plans(
    ListPlansRequest {
        page: Some(1),
        page_size: Some(20),
        product_id: None,
    }
).await?;
```
