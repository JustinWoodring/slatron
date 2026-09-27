-- Rebuild the permissions table so resource_type also allows 'content'
-- (SQLite cannot alter a CHECK constraint in place).
CREATE TABLE permissions_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    resource_type VARCHAR(50) NOT NULL CHECK(resource_type IN ('content', 'node', 'schedule')),
    resource_id INTEGER NOT NULL,
    permission_level VARCHAR(50) NOT NULL CHECK(permission_level IN ('read', 'write', 'admin')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO permissions_new SELECT * FROM permissions;

DROP TABLE permissions;
ALTER TABLE permissions_new RENAME TO permissions;
CREATE INDEX idx_permissions_user_id ON permissions(user_id);
