# List Orders

List all orders with optional filters.

```rust
let orders = clickpesa.orders().list(
    ListOrdersRequest {
        page: Some(1),
        page_size: Some(20),
        customer_id: None,
        status: None,
    }
).await?;
```
