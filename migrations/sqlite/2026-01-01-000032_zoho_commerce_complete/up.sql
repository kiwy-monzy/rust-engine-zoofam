-- Zoho Commerce complete — DEPRECATED.
-- Zoho-style tables (organizations, taxes, contacts, items, documents, etc.)
-- are now part of the unified commerce_erp (030) and crm_core (031) migrations
-- with money in minor units, multi-tenancy via org_id, and Contact 360.
-- This folder is kept only so the migration count and order stay stable.
-- We mark this migration as applied by creating then immediately dropping a
-- marker table. This ensures diesel_migrations::embed_migrations! treats
-- it as a valid migration.
CREATE TABLE _zoho_commerce_deprecated (id TEXT PRIMARY KEY, removed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')));