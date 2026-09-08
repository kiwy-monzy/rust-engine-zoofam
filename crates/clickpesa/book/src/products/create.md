# Create Product

Create a new product.

```rust
let product = clickpesa.products().create(
    CreateProductRequest {
        name: "Premium Plan".to_string(),
        description: Some("Premium subscription plan".to_string()),
        type_: "SERVICE".to_string(),
        category: Some("SOFTWARE".to_string()),
        image_url: Some("https://example.com/product.png".to_string()),
        home_url: Some("https://example.com".to_string()),
    }
).await?;
```
