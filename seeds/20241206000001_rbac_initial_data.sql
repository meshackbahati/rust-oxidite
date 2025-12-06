-- Seed: RBAC Initial Data
-- Created at: 2024-12-06T09:12:00Z

-- Create default roles
INSERT INTO roles (id, name, description, created_at, updated_at) VALUES 
(1, 'admin', 'Administrator with full access', strftime('%s', 'now'), strftime('%s', 'now')),
(2, 'user', 'Regular user with basic permissions', strftime('%s', 'now'), strftime('%s', 'now')),
(3, 'guest', 'Guest user with read-only access', strftime('%s', 'now'), strftime('%s', 'now'));

-- Create default permissions
INSERT INTO permissions (id, name, resource, action, description, created_at, updated_at) VALUES 
(1, 'users.create', 'users', 'create', 'Create new users', strftime('%s', 'now'), strftime('%s', 'now')),
(2, 'users.read', 'users', 'read', 'View users', strftime('%s', 'now'), strftime('%s', 'now')),
(3, 'users.update', 'users', 'update', 'Update users', strftime('%s', 'now'), strftime('%s', 'now')),
(4, 'users.delete', 'users', 'delete', 'Delete users', strftime('%s', 'now'), strftime('%s', 'now')),
(5, 'posts.create', 'posts', 'create', 'Create posts', strftime('%s', 'now'), strftime('%s', 'now')),
(6, 'posts.read', 'posts', 'read', 'View posts', strftime('%s', 'now'), strftime('%s', 'now')),
(7, 'posts.update', 'posts', 'update', 'Update posts', strftime('%s', 'now'), strftime('%s', 'now')),
(8, 'posts.delete', 'posts', 'delete', 'Delete posts', strftime('%s', 'now'), strftime('%s', 'now'));

-- Assign permissions to admin role (all permissions)
INSERT INTO role_permissions (role_id, permission_id) VALUES 
(1, 1), (1, 2), (1, 3), (1, 4),
(1, 5), (1, 6), (1, 7), (1, 8);

-- Assign permissions to user role (read/write for posts, read for users)
INSERT INTO role_permissions (role_id, permission_id) VALUES 
(2, 2), (2, 5), (2, 6), (2, 7);

-- Assign permissions to guest role (read-only)
INSERT INTO role_permissions (role_id, permission_id) VALUES 
(3, 2), (3, 6);
