-- Auth migration rollback
DROP TABLE IF EXISTS gateway_revoked_tokens CASCADE;
DROP TABLE IF EXISTS gateway_profiles CASCADE;
DROP TABLE IF EXISTS gateway_password_resets CASCADE;
DROP TABLE IF EXISTS gateway_sessions CASCADE;
DROP TABLE IF EXISTS gateway_users CASCADE;
