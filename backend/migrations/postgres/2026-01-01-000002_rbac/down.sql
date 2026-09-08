-- RBAC migration rollback
DROP TABLE IF EXISTS gateway_user_roles CASCADE;
DROP TABLE IF EXISTS gateway_role_permissions CASCADE;
DROP TABLE IF EXISTS gateway_permissions CASCADE;
DROP TABLE IF EXISTS gateway_roles CASCADE;
