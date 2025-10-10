-- Add roles and soft-delete support
PRAGMA foreign_keys = ON;

-- Soft delete columns on maps
ALTER TABLE maps ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0;
ALTER TABLE maps ADD COLUMN deleted_at TEXT;
ALTER TABLE maps ADD COLUMN deleted_by TEXT;

-- Moderator role on users
ALTER TABLE users ADD COLUMN is_moderator INTEGER NOT NULL DEFAULT 0;
