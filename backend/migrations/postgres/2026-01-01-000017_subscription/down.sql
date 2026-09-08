-- Subscription migration rollback
DROP TABLE IF EXISTS subscription_invoices CASCADE;
DROP TABLE IF EXISTS organization_subscriptions CASCADE;
DROP TABLE IF EXISTS subscription_plans CASCADE;
