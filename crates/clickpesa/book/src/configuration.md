# Configuration

## Environment Variables

Set these environment variables or pass them directly:

```bash
CLICKPESA_API_KEY=your_api_key
CLICKPESA_SECRET_KEY=your_secret_key
CLICKPESA_SANDBOX=true
```

## Programmatic Configuration

```rust
use clickpesa::ClickPesa;

let clickpesa = ClickPesa::new(
    "your_api_key".to_string(),
    "your_secret_key".to_string(),
    true, // sandbox mode
);
```

## From Environment

```rust
let clickpesa = ClickPesa::from_env()?;
```
