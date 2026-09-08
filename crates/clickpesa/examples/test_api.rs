use clickpesa::ClickPesa;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clickpesa = ClickPesa::from_env()?;
    
    println!("=== Testing Clickpesa API ===\n");
    
    // Generate unique order reference based on timestamp
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let order_ref = format!("ORD{}", timestamp);
    let bill_ref = format!("BILL{}", timestamp);
    
    // Test 1: Account Balance
    println!("1. Testing Account Balance...");
    match clickpesa.account().get_balance().await {
        Ok(balance) => println!("   Balance: {}", serde_json::to_string_pretty(&balance).unwrap()),
        Err(e) => println!("   Error: {}", e),
    }
    println!();

    // Test 2: Get Banks List
    println!("2. Testing Get Banks List...");
    match clickpesa.payouts().get_banks().await {
        Ok(_banks) => {
            println!("   Banks retrieved successfully");
        },
        Err(e) => println!("   Error: {}", e),
    }
    println!();

    // Test 3: Get Exchange Rates
    println!("3. Testing Exchange Rates...");
    match clickpesa.exchange().get_rates(Some("USD"), Some("TZS")).await {
        Ok(rates) => println!("   Rates: {}", serde_json::to_string_pretty(&rates).unwrap()),
        Err(e) => println!("   Error: {}", e),
    }
    println!();

    // Test 4: Preview USSD Push with Halotel
    println!("4. Testing Preview USSD Push (Halotel 255622194193)...");
    let halotel_order = format!("HAL{}", timestamp);
    match clickpesa.payments().preview_ussd_push("5000", &halotel_order, Some("255622194193"), "TZS", true).await {
        Ok(preview) => {
            println!("   Preview: {}", serde_json::to_string_pretty(&preview).unwrap());
        },
        Err(e) => println!("   Error: {}", e),
    }
    println!();

    // Test 5: Initiate USSD Push with Halotel
    println!("5. Testing Initiate USSD Push (Halotel 255622194193)...");
    let halotel_ussd = format!("HLT{}", timestamp);
    match clickpesa.payments().initiate_ussd_push("5000", "255622194193", &halotel_ussd, "TZS").await {
        Ok(result) => {
            println!("   SUCCESS! Result: {}", serde_json::to_string_pretty(&result).unwrap());
        },
        Err(e) => println!("   Error: {}", e),
    }
    println!();



    // Test 8: Lipa Namba Providers
    println!("8. Testing Lipa Namba Providers...");
    match clickpesa.payouts().list_lipa_namba_providers().await {
        Ok(providers) => println!("   Providers: {}", serde_json::to_string_pretty(&providers).unwrap()),
        Err(e) => println!("   Error: {}", e),
    }
    println!();

    // Test 9: Create BillPay Order Control Number
    println!("9. Testing BillPay Create Order Control Number...");
    match clickpesa.billpay().create_order_control_number(
        Some(&bill_ref),
        Some(5000.0),
        Some("Test Bill"),
        Some("ALLOW_PARTIAL_AND_OVER_PAYMENT"),
    ).await {
        Ok(result) => println!("   Result: {}", serde_json::to_string_pretty(&result).unwrap()),
        Err(e) => println!("   Error: {}", e),
    }
    println!();

    // Test 10: Generate Checkout Link
    println!("10. Testing Generate Checkout Link...");
    let checkout_order = format!("CHK{}", timestamp);
    match clickpesa.links().generate_checkout(
        "10000",
        &checkout_order,
        "TZS",
        Some("Test Payment"),
        Some("John Doe"),
        Some("test@example.com"),
        Some("255676544740"),
        Some("https://example.com/return"),
        None,
    ).await {
        Ok(result) => println!("   Link: {}", serde_json::to_string_pretty(&result).unwrap()),
        Err(e) => println!("   Error: {}", e),
    }
    println!();

    println!("=== Tests Complete ===");
    println!("Order refs used: {}, {}, {}", &halotel_order, &bill_ref, &checkout_order);
    Ok(())
}