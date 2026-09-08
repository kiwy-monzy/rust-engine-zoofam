-- ================================================================
-- RBAC: Roles, Permissions, Role Permissions, User Roles
-- ================================================================

CREATE TABLE gateway_roles (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(64)  NOT NULL UNIQUE,
    description VARCHAR(255) NOT NULL DEFAULT ''
);
INSERT INTO gateway_roles (name, description) VALUES
    ('admin',  'Full access'),
    ('viewer', 'Read-only access'),
    ('employee', 'Employee access');

CREATE TABLE gateway_permissions (
    id          SERIAL PRIMARY KEY,
    module      VARCHAR(64)  NOT NULL,
    action      VARCHAR(64)  NOT NULL,
    description VARCHAR(255) NOT NULL DEFAULT ''
);
CREATE UNIQUE INDEX gateway_permissions_module_action_key ON gateway_permissions (module, action);

INSERT INTO gateway_permissions (module, action, description) VALUES
    ('users',       'read',  'List and view users'),
    ('users',       'write', 'Create, update and delete users'),
    ('roles',       'read',  'List and view roles'),
    ('roles',       'write', 'Create and delete roles'),
    ('permissions', 'read',  'List and view permissions'),
    ('permissions', 'write', 'Create and delete permissions'),
    ('system',      'read',  'Read system settings'),
    ('system',      'write', 'Update system settings'),
    ('releases',    'read',  'List and view releases'),
    ('releases',    'write', 'Create and delete releases'),
    ('support',     'read',  'List and view support tickets'),
    ('support',     'write', 'Create and update support tickets'),
    ('wallet',      'read',  'View wallet passes'),
    ('wallet',      'write', 'Create and manage wallet passes'),
    ('maps',        'read',  'View maps'),
    ('maps',        'write', 'Edit maps'),
    ('maps',        'tiles', 'Access and generate map tiles'),
    ('fleet',       'read',  'View fleet'),
    ('fleet',       'write', 'Manage fleet'),
    ('storage',     'read',  'View storage'),
    ('storage',     'write', 'Upload and delete files'),
    ('erp',         'read',  'View ERP data'),
    ('erp',         'write', 'Manage ERP data'),
    ('erp',         'admin', 'Admin ERP'),
    ('crm',         'read',  'View CRM data'),
    ('crm',         'write', 'Manage CRM data'),
    ('crm',         'admin', 'Admin CRM'),
    ('dmc',         'read',  'View DMC data'),
    ('dmc',         'write', 'Create, update and delete DMC records'),
    ('dmc',         'admin', 'Manage DMC trips, quotes and bookings'),
    ('marketplace', 'read',  'View marketplace'),
    ('marketplace', 'write', 'Create, update and delete marketplace records'),
    ('marketplace', 'admin', 'Admin marketplace'),
    ('website',     'read',  'View website'),
    ('website',     'write', 'Manage website'),
    ('website',     'admin', 'Admin website'),
    ('profile',     'read',  'View profile'),
    ('profile',     'write', 'Update profile'),
    ('subscription','read',  'View subscription plans'),
    ('subscription','write', 'Manage subscriptions'),
    ('subscription','admin', 'Admin subscriptions'),
    ('thirdparty',  'read',  'View thirdparty providers'),
    ('thirdparty',  'write', 'Manage thirdparty providers');

CREATE TABLE gateway_role_permissions (
    role_id       INTEGER NOT NULL REFERENCES gateway_roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES gateway_permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE gateway_user_roles (
    user_id    VARCHAR(36) NOT NULL REFERENCES gateway_users(id) ON DELETE CASCADE,
    role_id    INTEGER     NOT NULL REFERENCES gateway_roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ   NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, role_id)
);
CREATE INDEX gateway_user_roles_user_id_idx ON gateway_user_roles(user_id);
CREATE INDEX gateway_user_roles_role_id_idx ON gateway_user_roles(role_id);

-- Grant admin role all permissions
INSERT INTO gateway_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM gateway_roles r CROSS JOIN gateway_permissions p WHERE r.name = 'admin';

-- Grant employee role read permissions
INSERT INTO gateway_role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM gateway_roles r JOIN gateway_permissions p ON p.action = 'read' WHERE r.name = 'employee';
