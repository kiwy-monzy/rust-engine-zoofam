-- Zoho Commerce complete — DEPRECATED.
-- Zoho-style tables (organizations, taxes, contacts, items, documents, etc.)
-- are now part of the unified commerce_erp (030) and crm_core (031) migrations
-- with money in minor units, multi-tenancy via org_id, and Contact 360.
-- This folder is kept only so the migration count and order stay stable.
-- The original zoho_* tables are NOT created here; this is a no-op marker
-- so diesel_migrations::embed_migrations! picks it up as a valid migration.
-- SQLite needs at least one statement; we use a benign one.
PRAGMA user_version = 32;