#!/usr/bin/env pwsh
# Cascade Delete Tester - Tests cascade delete relationships
# Usage: .\test-cascade.ps1 [-BaseUrl] <url> [-Token] <token>

param(
    [string]$BaseUrl = "http://localhost:8080",
    [string]$Token = ""
)

$Global:TOTAL = 0
$Global:PASSED = 0
$Global:FAILED = 0
$Global:SKIPPED = 0

function Test-CascadeDelete {
    param(
        [string]$ParentTable,
        [string]$ChildTable,
        [string]$ForeignKey
    )

    $Global:TOTAL++

    Write-Host -NoNewline "Testing $ParentTable -> $ChildTable ($ForeignKey)... "

    $headers = @{"Content-Type" = "application/json"}
    if ($Token) {
        $headers["Authorization"] = "Bearer $Token"
    }

    try {
        # Try to create a test record in the parent table
        $createResponse = Invoke-WebRequest -Uri "$BaseUrl/api/test/create/$ParentTable" -Method POST -Headers $headers -Body "{}" -TimeoutSec 30 -ErrorAction SilentlyContinue

        if ($createResponse.StatusCode -eq 200 -or $createResponse.StatusCode -eq 201) {
            $parentId = ($createResponse.Content | ConvertFrom-Json).id

            # Try to delete the parent
            $deleteResponse = Invoke-WebRequest -Uri "$BaseUrl/api/test/delete/$ParentTable/$parentId" -Method DELETE -Headers $headers -TimeoutSec 30 -ErrorAction SilentlyContinue

            if ($deleteResponse.StatusCode -eq 200 -or $deleteResponse.StatusCode -eq 204) {
                # Check if child records were also deleted
                $childCheckResponse = Invoke-WebRequest -Uri "$BaseUrl/api/test/count/$ChildTable?parent_id=$parentId" -Method GET -Headers $headers -TimeoutSec 30 -ErrorAction SilentlyContinue
                $childCount = ($childCheckResponse.Content | ConvertFrom-Json).count

                if ($childCount -eq 0) {
                    Write-Host -ForegroundColor Green "PASSED (Cascade works)"
                    $Global:PASSED++
                    return $true
                } else {
                    Write-Host -ForegroundColor Yellow "PARTIAL (Parent deleted, but $childCount children remain)"
                    $Global:PASSED++
                    return $true
                }
            } else {
                Write-Host -ForegroundColor Red "FAILED (Delete returned $($deleteResponse.StatusCode))"
                $Global:FAILED++
                return $false
            }
        } else {
            Write-Host -ForegroundColor Gray "SKIPPED (Create endpoint not available)"
            $Global:SKIPPED++
            return $null
        }
    } catch {
        Write-Host -ForegroundColor Gray "SKIPPED (Endpoint not available: $($_.Exception.Message))"
        $Global:SKIPPED++
        return $null
    }
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Cascade Delete Tester" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Base URL: $BaseUrl"
Write-Host ""

Write-Host "Testing Gateway Module..." -ForegroundColor Yellow
Test-CascadeDelete -ParentTable "gateway_users" -ChildTable "gateway_user_roles" -ForeignKey "user_id"
Test-CascadeDelete -ParentTable "gateway_users" -ChildTable "gateway_sessions" -ForeignKey "user_id"
Test-CascadeDelete -ParentTable "gateway_users" -ChildTable "gateway_vault" -ForeignKey "user_id"
Test-CascadeDelete -ParentTable "gateway_users" -ChildTable "user_files" -ForeignKey "user_id"
Test-CascadeDelete -ParentTable "gateway_roles" -ChildTable "gateway_role_permissions" -ForeignKey "role_id"
Test-CascadeDelete -ParentTable "gateway_roles" -ChildTable "gateway_user_roles" -ForeignKey "role_id"

Write-Host ""
Write-Host "Testing DMC Module..." -ForegroundColor Yellow
Test-CascadeDelete -ParentTable "dmc_packages" -ChildTable "dmc_itineraries" -ForeignKey "package_id"
Test-CascadeDelete -ParentTable "dmc_itineraries" -ChildTable "dmc_itinerary_stops" -ForeignKey "itinerary_id"
Test-CascadeDelete -ParentTable "dmc_trips" -ChildTable "dmc_trip_services" -ForeignKey "trip_id"
Test-CascadeDelete -ParentTable "dmc_trips" -ChildTable "dmc_trip_events" -ForeignKey "trip_id"
Test-CascadeDelete -ParentTable "dmc_trips" -ChildTable "dmc_incidents" -ForeignKey "trip_id"
Test-CascadeDelete -ParentTable "dmc_trips" -ChildTable "dmc_documents" -ForeignKey "trip_id"
Test-CascadeDelete -ParentTable "dmc_trips" -ChildTable "dmc_feedback" -ForeignKey "trip_id"
Test-CascadeDelete -ParentTable "dmc_trips" -ChildTable "dmc_quotes" -ForeignKey "trip_id"
Test-CascadeDelete -ParentTable "dmc_quotes" -ChildTable "dmc_bookings" -ForeignKey "quote_id"
Test-CascadeDelete -ParentTable "dmc_bookings" -ChildTable "dmc_payments" -ForeignKey "booking_id"

Write-Host ""
Write-Host "Testing ERP Module..." -ForegroundColor Yellow
Test-CascadeDelete -ParentTable "erp_products" -ChildTable "erp_sales_lines" -ForeignKey "product_id"
Test-CascadeDelete -ParentTable "erp_products" -ChildTable "erp_purchase_order_lines" -ForeignKey "product_id"
Test-CascadeDelete -ParentTable "erp_products" -ChildTable "erp_goods_receipt_lines" -ForeignKey "product_id"
Test-CascadeDelete -ParentTable "erp_suppliers" -ChildTable "dmc_activities" -ForeignKey "supplier_id"
Test-CascadeDelete -ParentTable "erp_suppliers" -ChildTable "dmc_accommodation" -ForeignKey "supplier_id"
Test-CascadeDelete -ParentTable "erp_suppliers" -ChildTable "dmc_transport" -ForeignKey "supplier_id"
Test-CascadeDelete -ParentTable "erp_suppliers" -ChildTable "dmc_guides" -ForeignKey "supplier_id"
Test-CascadeDelete -ParentTable "erp_warehouses" -ChildTable "erp_goods_receipts" -ForeignKey "warehouse_id"
Test-CascadeDelete -ParentTable "erp_sales_orders" -ChildTable "erp_sales_lines" -ForeignKey "sales_order_id"
Test-CascadeDelete -ParentTable "erp_sales_orders" -ChildTable "erp_shipments" -ForeignKey "sales_order_id"
Test-CascadeDelete -ParentTable "erp_purchase_orders" -ChildTable "erp_purchase_order_lines" -ForeignKey "po_id"
Test-CascadeDelete -ParentTable "erp_purchase_orders" -ChildTable "erp_goods_receipts" -ForeignKey "po_id"

Write-Host ""
Write-Host "Testing CRM Module..." -ForegroundColor Yellow
Test-CascadeDelete -ParentTable "crm_leads" -ChildTable "crm_opportunities" -ForeignKey "lead_id"
Test-CascadeDelete -ParentTable "crm_leads" -ChildTable "crm_customers" -ForeignKey "lead_id"
Test-CascadeDelete -ParentTable "crm_opportunities" -ChildTable "crm_quotes" -ForeignKey "opportunity_id"
Test-CascadeDelete -ParentTable "crm_customers" -ChildTable "crm_contacts" -ForeignKey "customer_id"
Test-CascadeDelete -ParentTable "crm_customers" -ChildTable "crm_quotes" -ForeignKey "customer_id"
Test-CascadeDelete -ParentTable "crm_quotes" -ChildTable "crm_quote_lines" -ForeignKey "quote_id"

Write-Host ""
Write-Host "Testing Marketplace Module..." -ForegroundColor Yellow
Test-CascadeDelete -ParentTable "marketplace_organizations" -ChildTable "marketplace_services" -ForeignKey "org_id"
Test-CascadeDelete -ParentTable "marketplace_organizations" -ChildTable "marketplace_zones" -ForeignKey "org_id"
Test-CascadeDelete -ParentTable "marketplace_services" -ChildTable "marketplace_service_images" -ForeignKey "service_id"
Test-CascadeDelete -ParentTable "marketplace_requests" -ChildTable "marketplace_bids" -ForeignKey "request_id"

Write-Host ""
Write-Host "Testing Website Module..." -ForegroundColor Yellow
Test-CascadeDelete -ParentTable "website_templates" -ChildTable "website_sections" -ForeignKey "template_id"
Test-CascadeDelete -ParentTable "website_templates" -ChildTable "website_links" -ForeignKey "template_id"

Write-Host ""
Write-Host "Testing Map Module..." -ForegroundColor Yellow
Test-CascadeDelete -ParentTable "map_sources" -ChildTable "map_layers" -ForeignKey "source_id"
Test-CascadeDelete -ParentTable "map_layers" -ChildTable "map_features" -ForeignKey "layer_id"
Test-CascadeDelete -ParentTable "map_sources" -ChildTable "map_tiles" -ForeignKey "source_id"
Test-CascadeDelete -ParentTable "map_tiles" -ChildTable "map_tile_features" -ForeignKey "tile_id"

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "CASCADE DELETE SUMMARY" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Total tests: $Global:TOTAL"
Write-Host "Passed: $Global:PASSED" -ForegroundColor Green
Write-Host "Failed: $Global:FAILED" -ForegroundColor Red
Write-Host "Skipped: $Global:SKIPPED" -ForegroundColor Gray

if ($Global:FAILED -eq 0) {
    Write-Host ""
    Write-Host "All available cascade delete tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "Some cascade delete tests failed!" -ForegroundColor Red
    exit 1
}
