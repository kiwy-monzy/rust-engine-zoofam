# Create Customer

Create a new customer.

```rust
let customer = clickpesa.customers().create(
    CreateCustomerRequest {
        email: "customer@example.com".to_string(),
        name: "John Doe".to_string(),
        phone: Some("+254712345678".to_string()),
        address: Some(Address {
            line1: "123 Main Street".to_string(),
            line2: None,
            city: "Nairobi".to_string(),
            state: Some("Nairobi".to_string()),
            postal_code: Some("00100".to_string()),
            country: "KE".to_string(),
        }),
    }
).await?;
```
