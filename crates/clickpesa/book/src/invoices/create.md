# Create Invoice

Create a new invoice.

```rust
let invoice = clickpesa.invoices().create(
    CreateInvoiceRequest {
        number: "INV-2024-001".to_string(),
        customer_id: "CUSTOMER_ID".to_string(),
        amount: Money::from_f64(100.00, "KES"),
        currency: "KES".to_string(),
        due_date: "2024-12-31".to_string(),
        issued_date: "2024-01-01".to_string(),
        line_items: vec![
            LineItemInput {
                name: "Service A".to_string(),
                description: Some("Description of service".to_string()),
                quantity: "1".to_string(),
                unit_amount: Money::from_f64(50.00, "KES"),
            },
            LineItemInput {
                name: "Service B".to_string(),
                description: None,
                quantity: "1".to_string(),
                unit_amount: Money::from_f64(50.00, "KES"),
            },
        ],
        note: Some("Thank you for your business".to_string()),
    }
).await?;
```
