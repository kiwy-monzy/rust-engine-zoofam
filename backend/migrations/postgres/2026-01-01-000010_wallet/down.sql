-- Wallet migration rollback
DROP TABLE IF EXISTS dmc_wallet_registrations CASCADE;
DROP TABLE IF EXISTS dmc_wallet_passes CASCADE;
