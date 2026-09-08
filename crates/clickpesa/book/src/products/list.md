# List Products

List all products.

```rust
let products = clickpesa.products().list(
    ListProductsRequest {
        page: Some(1),
        page_size: Some(20),
    }
).await?;
```
