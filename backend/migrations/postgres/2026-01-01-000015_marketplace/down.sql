-- Marketplace migration rollback
DROP TABLE IF EXISTS marketplace_notifications CASCADE;
DROP TABLE IF EXISTS marketplace_promotions CASCADE;
DROP TABLE IF EXISTS marketplace_payouts CASCADE;
DROP TABLE IF EXISTS marketplace_commissions CASCADE;
DROP TABLE IF EXISTS marketplace_reviews CASCADE;
DROP TABLE IF EXISTS marketplace_cart CASCADE;
DROP TABLE IF EXISTS marketplace_bids CASCADE;
DROP TABLE IF EXISTS marketplace_requests CASCADE;
DROP TABLE IF EXISTS marketplace_bookings CASCADE;
DROP TABLE IF EXISTS marketplace_service_images CASCADE;
DROP TABLE IF EXISTS marketplace_zones CASCADE;
DROP TABLE IF EXISTS marketplace_services CASCADE;
DROP TABLE IF EXISTS marketplace_categories CASCADE;
DROP TABLE IF EXISTS marketplace_org_members CASCADE;
DROP TABLE IF EXISTS marketplace_organizations CASCADE;
