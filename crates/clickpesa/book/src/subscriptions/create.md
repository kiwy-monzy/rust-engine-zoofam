# Create Subscription

Create a new subscription.

```rust
let subscription = clickpesa.subscriptions().create(
    CreateSubscriptionRequest {
        plan_id: "PLAN_ID".to_string(),
        customer_id: "CUSTOMER_ID".to_string(),
        start_time: Some("2024-12-01T00:00:00Z".to_string()),
        quantity: Some("1".to_string()),
        shipping_amount: None,
        application_context: Some(ApplicationContext {
            brand_name: Some("My Company".to_string()),
            locale: Some("en-US".to_string()),
            shipping_preference: Some("SET_PROVIDED_ADDRESS".to_string()),
            user_action: Some("SUBSCRIBE_NOW".to_string()),
            return_url: Some("https://example.com/success".to_string()),
            cancel_url: Some("https://example.com/cancel".to_string()),
        }),
    }
).await?;
```
