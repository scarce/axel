-- Migration: Add automerge_doc columns for CRDT-based conflict-free synchronization
-- This migration adds BYTEA columns to store Automerge document bytes for all synced entities

-- Add automerge_doc column to tasks table
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS automerge_doc BYTEA;

-- Add automerge_doc column to workspaces table
ALTER TABLE workspaces ADD COLUMN IF NOT EXISTS automerge_doc BYTEA;

-- Add automerge_doc column to skills table
ALTER TABLE skills ADD COLUMN IF NOT EXISTS automerge_doc BYTEA;

-- Add automerge_doc column to contexts table
ALTER TABLE contexts ADD COLUMN IF NOT EXISTS automerge_doc BYTEA;

-- Add automerge_doc column to terminals table
ALTER TABLE terminals ADD COLUMN IF NOT EXISTS automerge_doc BYTEA;

-- Add automerge_doc column to hints table
ALTER TABLE hints ADD COLUMN IF NOT EXISTS automerge_doc BYTEA;

-- Add automerge_doc column to organizations table
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS automerge_doc BYTEA;

-- Create index on tasks for faster automerge doc queries (optional, for large tables)
CREATE INDEX IF NOT EXISTS idx_tasks_automerge_doc_not_null
    ON tasks (id)
    WHERE automerge_doc IS NOT NULL;

-- Create index on workspaces for faster automerge doc queries
CREATE INDEX IF NOT EXISTS idx_workspaces_automerge_doc_not_null
    ON workspaces (id)
    WHERE automerge_doc IS NOT NULL;

-- Comment the columns for documentation
COMMENT ON COLUMN tasks.automerge_doc IS 'Automerge CRDT document bytes for conflict-free sync';
COMMENT ON COLUMN workspaces.automerge_doc IS 'Automerge CRDT document bytes for conflict-free sync';
COMMENT ON COLUMN skills.automerge_doc IS 'Automerge CRDT document bytes for conflict-free sync';
COMMENT ON COLUMN contexts.automerge_doc IS 'Automerge CRDT document bytes for conflict-free sync';
COMMENT ON COLUMN terminals.automerge_doc IS 'Automerge CRDT document bytes for conflict-free sync';
COMMENT ON COLUMN hints.automerge_doc IS 'Automerge CRDT document bytes for conflict-free sync';
COMMENT ON COLUMN organizations.automerge_doc IS 'Automerge CRDT document bytes for conflict-free sync';
