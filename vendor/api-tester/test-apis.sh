#!/bin/bash
# API Curl Tester - Tests all API endpoints
# Usage: ./test-apis.sh <base_url> <token>

set -e

BASE_URL="${1:-http://localhost:8080}"
TOKEN="${2:-}"
ITERATIONS="${3:-5}"

echo "=========================================="
echo "API Curl Tester"
echo "=========================================="
echo "Base URL: $BASE_URL"
echo "Iterations: $ITERATIONS"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counters
TOTAL=0
PASSED=0
FAILED=0

# Test function
test_api() {
    local method=$1
    local endpoint=$2
    local description=$3
    local data=$4

    TOTAL=$((TOTAL + 1))

    echo -n "Testing $method $endpoint... "

    if [ "$method" = "GET" ]; then
        if [ -n "$TOKEN" ]; then
            response=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $TOKEN" "$BASE_URL$endpoint")
        else
            response=$(curl -s -w "\n%{http_code}" "$BASE_URL$endpoint")
        fi
    else
        if [ -n "$TOKEN" ]; then
            if [ -n "$data" ]; then
                response=$(curl -s -w "\n%{http_code}" -X "$method" -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d "$data" "$BASE_URL$endpoint")
            else
                response=$(curl -s -w "\n%{http_code}" -X "$method" -H "Authorization: Bearer $TOKEN" "$BASE_URL$endpoint")
            fi
        else
            if [ -n "$data" ]; then
                response=$(curl -s -w "\n%{http_code}" -X "$method" -H "Content-Type: application/json" -d "$data" "$BASE_URL$endpoint")
            else
                response=$(curl -s -w "\n%{http_code}" -X "$method" "$BASE_URL$endpoint")
            fi
        fi
    fi

    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    if [ "$status_code" -ge 200 ] && [ "$status_code" -lt 300 ]; then
        echo -e "${GREEN}OK${NC} (${status_code})"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAILED${NC} (${status_code})"
        FAILED=$((FAILED + 1))
    fi
}

echo "Testing System APIs..."
test_api "GET" "/api/system/health" "Health check"
test_api "GET" "/api/system/version" "Get version"
test_api "GET" "/api/system" "Get system info"

echo ""
echo "Testing Auth APIs..."
test_api "POST" "/api/auth/register" "Register" '{"email":"test@example.com","password":"Test@123456","username":"testuser","first_name":"Test","last_name":"User","display_name":"Test User"}'
test_api "POST" "/api/auth/login" "Login" '{"email":"admin@example.com","password":"admin12345"}'
test_api "GET" "/api/auth/me" "Get current user"

echo ""
echo "Testing RBAC APIs..."
test_api "GET" "/api/rbac/users" "List users"
test_api "GET" "/api/rbac/roles" "List roles"
test_api "GET" "/api/rbac/permissions" "List permissions"

echo ""
echo "Testing CRM APIs..."
test_api "GET" "/api/crm/leads" "List leads"
test_api "GET" "/api/crm/customers" "List customers"
test_api "GET" "/api/crm/opportunities" "List opportunities"
test_api "GET" "/api/crm/quotes" "List quotes"

echo ""
echo "Testing ERP APIs..."
test_api "GET" "/api/erp/products" "List products"
test_api "GET" "/api/erp/suppliers" "List suppliers"
test_api "GET" "/api/erp/warehouses" "List warehouses"
test_api "GET" "/api/erp/sales-orders" "List sales orders"
test_api "GET" "/api/erp/purchase-orders" "List purchase orders"

echo ""
echo "Testing DMC APIs..."
test_api "GET" "/api/dmc/trips" "List trips"
test_api "GET" "/api/dmc/packages" "List packages"
test_api "GET" "/api/dmc/destinations" "List destinations"
test_api "GET" "/api/dmc/activities" "List activities"
test_api "GET" "/api/dmc/accommodation" "List accommodation"
test_api "GET" "/api/dmc/transport" "List transport"

echo ""
echo "Testing Marketplace APIs..."
test_api "GET" "/api/marketplace/services" "List services"
test_api "GET" "/api/marketplace/categories" "List categories"
test_api "GET" "/api/marketplace/organizations" "List organizations"
test_api "GET" "/api/marketplace/bookings" "List bookings"

echo ""
echo "Testing Website APIs..."
test_api "GET" "/api/website/templates" "List templates"
test_api "GET" "/api/website/sections" "List sections"
test_api "GET" "/api/website/links" "List links"

echo ""
echo "Testing Storage APIs..."
test_api "GET" "/api/storage/files" "List files"

echo ""
echo "Testing Map APIs..."
test_api "GET" "/api/maps/sources" "List map sources"
test_api "GET" "/api/maps/layers" "List map layers"
test_api "GET" "/api/maps/styles" "List map styles"

echo ""
echo "Testing Fleet APIs..."
test_api "GET" "/api/fleet/vehicles" "List vehicles"

echo ""
echo "Testing Wallet APIs..."
test_api "GET" "/api/wallet/passes" "List wallet passes"

echo ""
echo "Testing Subscription APIs..."
test_api "GET" "/api/subscription/plans" "List subscription plans"

echo ""
echo "=========================================="
echo "SUMMARY"
echo "=========================================="
echo -e "Total: $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}Some tests failed!${NC}"
    exit 1
fi
