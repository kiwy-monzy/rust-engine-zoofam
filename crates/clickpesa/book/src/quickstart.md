# Quick Start

## Basic Example

```rust
use clickpesa::{ClickPesa, models::Money};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clickpesa = ClickPesa::from_env()?;
    
    // Create an order
    let order = clickpesa.orders().create(
        Money::from_f64(100.00, "KES"),
        "Order description".to_string(),
        None,
    ).await?;
    
    println!("Created order: {}", order.id);
    Ok(())
}
```
