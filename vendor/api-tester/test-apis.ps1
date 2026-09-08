#!/usr/bin/env pwsh
# API Curl Tester - Tests all API endpoints (PowerShell version)
# Usage: .\test-apis.ps1 [-BaseUrl] <url> [-Token] <token> [-Iterations] <n>

param(
    [string]$BaseUrl = "http://localhost:8080",
    [string]$Token = "",
    [int]$Iterations = 5
)

$Global:TOTAL = 0
$Global:PASSED = 0
$Global:FAILED = 0

function Test-API {
    param(
        [string]$Method,
        [string]$Endpoint,
        [string]$Description,
        [string]$Data = ""
    )

    $Global:TOTAL++

    Write-Host -NoNewline "Testing $Method $Endpoint... "

    $headers = @{"Content-Type" = "application/json"}
    if ($Token) {
        $headers["Authorization"] = "Bearer $Token"
    }

    try {
        if ($Method -eq "GET") {
            $response = Invoke-WebRequest -Uri "$BaseUrl$Endpoint" -Method GET -Headers $headers -TimeoutSec 30
        } else {
            if ($Data) {
                $response = Invoke-WebRequest -Uri "$BaseUrl$Endpoint" -Method $Method -Headers $headers -Body $Data -TimeoutSec 30
            } else {
                $response = Invoke-WebRequest -Uri "$BaseUrl$Endpoint" -Method $Method -Headers $headers -TimeoutSec 30
            }
        }

        $statusCode = [int]$response.StatusCode

        if ($statusCode -ge 200 -and $statusCode -lt 300) {
            Write-Host -ForegroundColor Green "OK ($statusCode)"
            $Global:PASSED++
            return $true
        } else {
            Write-Host -ForegroundColor Red "FAILED ($statusCode)"
            $Global:FAILED++
            return $false
        }
    } catch {
        Write-Host -ForegroundColor Red "ERROR: $($_.Exception.Message)"
        $Global:FAILED++
        return $false
    }
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "API Curl Tester" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Base URL: $BaseUrl"
Write-Host "Iterations: $Iterations"
Write-Host ""

Write-Host "Testing System APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/system/health" -Description "Health check"
Test-API -Method "GET" -Endpoint "/api/system/version" -Description "Get version"
Test-API -Method "GET" -Endpoint "/api/system" -Description "Get system info"

Write-Host ""
Write-Host "Testing Auth APIs..." -ForegroundColor Yellow
Test-API -Method "POST" -Endpoint "/api/auth/register" -Description "Register" -Data '{"email":"test@example.com","password":"Test@123456","username":"testuser","first_name":"Test","last_name":"User","display_name":"Test User"}'
Test-API -Method "POST" -Endpoint "/api/auth/login" -Description "Login" -Data '{"email":"admin@example.com","password":"admin12345"}'
Test-API -Method "GET" -Endpoint "/api/auth/me" -Description "Get current user"

Write-Host ""
Write-Host "Testing RBAC APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/rbac/users" -Description "List users"
Test-API -Method "GET" -Endpoint "/api/rbac/roles" -Description "List roles"
Test-API -Method "GET" -Endpoint "/api/rbac/permissions" -Description "List permissions"

Write-Host ""
Write-Host "Testing CRM APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/crm/leads" -Description "List leads"
Test-API -Method "GET" -Endpoint "/api/crm/customers" -Description "List customers"
Test-API -Method "GET" -Endpoint "/api/crm/opportunities" -Description "List opportunities"
Test-API -Method "GET" -Endpoint "/api/crm/quotes" -Description "List quotes"

Write-Host ""
Write-Host "Testing ERP APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/erp/products" -Description "List products"
Test-API -Method "GET" -Endpoint "/api/erp/suppliers" -Description "List suppliers"
Test-API -Method "GET" -Endpoint "/api/erp/warehouses" -Description "List warehouses"
Test-API -Method "GET" -Endpoint "/api/erp/sales-orders" -Description "List sales orders"
Test-API -Method "GET" -Endpoint "/api/erp/purchase-orders" -Description "List purchase orders"

Write-Host ""
Write-Host "Testing DMC APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/dmc/trips" -Description "List trips"
Test-API -Method "GET" -Endpoint "/api/dmc/packages" -Description "List packages"
Test-API -Method "GET" -Endpoint "/api/dmc/destinations" -Description "List destinations"
Test-API -Method "GET" -Endpoint "/api/dmc/activities" -Description "List activities"
Test-API -Method "GET" -Endpoint "/api/dmc/accommodation" -Description "List accommodation"
Test-API -Method "GET" -Endpoint "/api/dmc/transport" -Description "List transport"

Write-Host ""
Write-Host "Testing Marketplace APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/marketplace/services" -Description "List services"
Test-API -Method "GET" -Endpoint "/api/marketplace/categories" -Description "List categories"
Test-API -Method "GET" -Endpoint "/api/marketplace/organizations" -Description "List organizations"
Test-API -Method "GET" -Endpoint "/api/marketplace/bookings" -Description "List bookings"

Write-Host ""
Write-Host "Testing Website APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/website/templates" -Description "List templates"
Test-API -Method "GET" -Endpoint "/api/website/sections" -Description "List sections"
Test-API -Method "GET" -Endpoint "/api/website/links" -Description "List links"

Write-Host ""
Write-Host "Testing Storage APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/storage/files" -Description "List files"

Write-Host ""
Write-Host "Testing Map APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/maps/sources" -Description "List map sources"
Test-API -Method "GET" -Endpoint "/api/maps/layers" -Description "List map layers"
Test-API -Method "GET" -Endpoint "/api/maps/styles" -Description "List map styles"

Write-Host ""
Write-Host "Testing Fleet APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/fleet/vehicles" -Description "List vehicles"

Write-Host ""
Write-Host "Testing Wallet APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/wallet/passes" -Description "List wallet passes"

Write-Host ""
Write-Host "Testing Subscription APIs..." -ForegroundColor Yellow
Test-API -Method "GET" -Endpoint "/api/subscription/plans" -Description "List subscription plans"

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "SUMMARY" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Total: $Global:TOTAL"
Write-Host "Passed: $Global:PASSED" -ForegroundColor Green
Write-Host "Failed: $Global:FAILED" -ForegroundColor Red

if ($Global:FAILED -eq 0) {
    Write-Host ""
    Write-Host "All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "Some tests failed!" -ForegroundColor Red
    exit 1
}
