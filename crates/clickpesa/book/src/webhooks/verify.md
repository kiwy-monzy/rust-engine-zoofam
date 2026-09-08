# Verify Webhook

Verify the authenticity of a webhook event.

```rust
use clickpesa::webhooks::{WebhookHeaders, verify_webhook_signature};

let headers = WebhookHeaders::from_headers(&request.headers());
let is_valid = verify_webhook_signature(
    &request.body(),
    &headers,
    "your_webhook_secret",
)?;

if is_valid {
    // Process the webhook event
} else {
    // Reject the request
}
```
