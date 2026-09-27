-- Restore the original resource_type CHECK (drops any 'content' grants).
CREATE TABLE permissions_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    resource_type VARCHAR(50) NOT NULL CHECK(resource_type IN ('node', 'schedule')),
    resource_id INTEGER NOT NULL,
    permission_level VARCHAR(50) NOT NULL CHECK(permission_level IN ('read', 'write', 'admin')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO permissions_old SELECT * FROM permissions WHERE resource_type IN ('node', 'schedule');

DROP TABLE permissions;
ALTER TABLE permissions_old RENAME TO permissions;
CREATE INDEX idx_permissions_user_id ON permissions(user_id);
