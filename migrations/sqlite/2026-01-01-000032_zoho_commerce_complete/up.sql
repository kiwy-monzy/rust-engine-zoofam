-- Zoho Commerce complete — DEPRECATED.
-- Zoho-style tables (organizations, taxes, contacts, items, documents, etc.)
-- are now part of the unified commerce_erp (030) and crm_core (031) migrations
-- with money in minor units, multi-tenancy via org_id, and Contact 360.
-- This folder is kept only so the migration count and order stay stable.
-- We need a real schema operation so diesel_migrations::embed_migrations!
-- counts this as a valid migration.
CREATE TABLE IF NOT EXISTS _zoho_legacy_marker (applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')));
DROP TABLE IF EXISTS _zoho_legacy_marker;