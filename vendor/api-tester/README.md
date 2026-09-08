# API Tester CLI

A comprehensive CLI tool for testing APIs based on Swagger/OpenAPI specifications.

## Features

- **OpenAPI Integration**: Automatically fetches API spec from running server
- **User Seeding**: Creates system admin, tester admin, and 100 test users
- **API Testing**: Tests all endpoints with configurable iterations
- **Cascade Delete Testing**: Verifies referential integrity across modules
- **Detailed Reporting**: Summary of pass/fail rates and response times

## Installation

```bash
cd vendor/api-tester
npm install
npm run build
```

## Usage

### Basic API Testing

```bash
# Test all APIs on local server (default)
npm start

# Test against custom URL
npm start -- --url http://production-api.example.com

# Run 5 iterations per endpoint (default)
npm start -- --iterations 10
```

### Seeding Test Data

```bash
# Seed users via API
npx tsx src/seed.ts

# Or with custom API URL
API_URL=http://localhost:8080 npx tsx src/seed.ts
```

### Using PowerShell Scripts

```powershell
# Test all APIs
.\test-apis.ps1 -BaseUrl "http://localhost:8080" -Token "your-jwt-token"

# Test with 5 iterations
.\test-apis.ps1 -BaseUrl "http://localhost:8080" -Iterations 5
```

### Testing Cascade Deletes

```powershell
.\test-cascade.ps1 -BaseUrl "http://localhost:8080" -Token "your-jwt-token"
```

## API Endpoints Tested

The tool tests the following modules:

- **System**: Health, version, system info
- **Auth**: Register, login, logout, refresh, me
- **RBAC**: Users, roles, permissions
- **CRM**: Leads, customers, opportunities, quotes
- **ERP**: Products, suppliers, warehouses, orders
- **DMC**: Trips, packages, destinations, activities
- **Marketplace**: Services, categories, organizations, bookings
- **Website**: Templates, sections, links
- **Storage**: Files
- **Maps**: Sources, layers, styles, features
- **Fleet**: Vehicles
- **Wallet**: Passes
- **Subscription**: Plans

## Architecture

```
src/
├── index.ts      # Main CLI entry point with OpenAPI parsing
├── seed.ts       # User seeding script
```

## Cascade Delete Relationships

The tool tests cascade deletes for the following relationships:

### Gateway Module
- gateway_users → gateway_user_roles (CASCADE)
- gateway_users → gateway_sessions (CASCADE)
- gateway_users → gateway_vault (CASCADE)
- gateway_users → user_files (CASCADE)
- gateway_roles → gateway_role_permissions (CASCADE)
- gateway_roles → gateway_user_roles (CASCADE)

### DMC Module
- dmc_packages → dmc_itineraries (CASCADE)
- dmc_itineraries → dmc_itinerary_stops (CASCADE)
- dmc_trips → dmc_trip_services (CASCADE)
- dmc_trips → dmc_trip_events (CASCADE)
- dmc_trips → dmc_incidents (CASCADE)
- dmc_trips → dmc_documents (CASCADE)
- dmc_trips → dmc_quotes (CASCADE)
- dmc_quotes → dmc_bookings (CASCADE)
- dmc_bookings → dmc_payments (CASCADE)

### ERP Module
- erp_products → erp_sales_lines (CASCADE)
- erp_products → erp_purchase_order_lines (CASCADE)
- erp_products → erp_goods_receipt_lines (CASCADE)
- erp_suppliers → dmc_activities (SET NULL)
- erp_suppliers → dmc_accommodation (SET NULL)
- erp_suppliers → dmc_transport (SET NULL)
- erp_warehouses → erp_goods_receipts (CASCADE)
- erp_sales_orders → erp_sales_lines (CASCADE)
- erp_sales_orders → erp_shipments (CASCADE)

### CRM Module
- crm_leads → crm_opportunities (SET NULL)
- crm_leads → crm_customers (SET NULL)
- crm_opportunities → crm_quotes (SET NULL)
- crm_customers → crm_contacts (CASCADE)
- crm_customers → crm_quotes (SET NULL)
- crm_quotes → crm_quote_lines (CASCADE)

### Marketplace Module
- marketplace_organizations → marketplace_services (CASCADE)
- marketplace_organizations → marketplace_zones (CASCADE)
- marketplace_services → marketplace_service_images (CASCADE)
- marketplace_requests → marketplace_bids (CASCADE)

### Website Module
- website_templates → website_sections (CASCADE)
- website_templates → website_links (CASCADE)

### Map Module
- map_sources → map_layers (CASCADE)
- map_layers → map_features (CASCADE)
- map_sources → map_tiles (CASCADE)
- map_tiles → map_tile_features (CASCADE)

## Notes

- Some endpoints require authentication. The tool will skip endpoints that fail authentication.
- Cascade delete tests require a running API server with test endpoints.
- The tool is designed for testing and development purposes.
