-- Migration: Add task_assignees junction table for many-to-many relationship
-- This replaces the single assigned_to foreign key

-- Create junction table for task assignees
CREATE TABLE IF NOT EXISTS task_assignees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    profile_id UUID NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID REFERENCES profiles(id) ON DELETE SET NULL,
    UNIQUE(task_id, profile_id)
);

-- Create indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_task_assignees_task_id ON task_assignees(task_id);
CREATE INDEX IF NOT EXISTS idx_task_assignees_profile_id ON task_assignees(profile_id);

-- Enable RLS
ALTER TABLE task_assignees ENABLE ROW LEVEL SECURITY;

-- RLS Policies
-- Users can view assignees for tasks they have access to
CREATE POLICY "Users can view task assignees"
    ON task_assignees FOR SELECT
    USING (
        EXISTS (
            SELECT 1 FROM tasks t
            WHERE t.id = task_assignees.task_id
            AND (
                t.created_by = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM task_assignees ta2
                    WHERE ta2.task_id = t.id AND ta2.profile_id = auth.uid()
                )
                OR EXISTS (
                    SELECT 1 FROM workspaces w
                    WHERE w.id = t.workspace_id
                    AND (
                        w.owner_id = auth.uid()
                        OR EXISTS (
                            SELECT 1 FROM organization_members om
                            WHERE om.organization_id = w.organization_id
                            AND om.user_id = auth.uid()
                        )
                    )
                )
            )
        )
    );

-- Users can add assignees to tasks they created or in workspaces they manage
CREATE POLICY "Users can add task assignees"
    ON task_assignees FOR INSERT
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM tasks t
            WHERE t.id = task_assignees.task_id
            AND (
                t.created_by = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM workspaces w
                    WHERE w.id = t.workspace_id
                    AND (
                        w.owner_id = auth.uid()
                        OR EXISTS (
                            SELECT 1 FROM organization_members om
                            WHERE om.organization_id = w.organization_id
                            AND om.user_id = auth.uid()
                            AND om.role IN ('admin', 'owner')
                        )
                    )
                )
            )
        )
    );

-- Users can remove assignees from tasks they created or manage
CREATE POLICY "Users can remove task assignees"
    ON task_assignees FOR DELETE
    USING (
        EXISTS (
            SELECT 1 FROM tasks t
            WHERE t.id = task_assignees.task_id
            AND (
                t.created_by = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM workspaces w
                    WHERE w.id = t.workspace_id
                    AND (
                        w.owner_id = auth.uid()
                        OR EXISTS (
                            SELECT 1 FROM organization_members om
                            WHERE om.organization_id = w.organization_id
                            AND om.user_id = auth.uid()
                            AND om.role IN ('admin', 'owner')
                        )
                    )
                )
            )
        )
    );

-- Optional: If you have an existing assigned_to column, migrate the data
-- DO $$
-- BEGIN
--     IF EXISTS (SELECT 1 FROM information_schema.columns
--                WHERE table_name = 'tasks' AND column_name = 'assigned_to') THEN
--         INSERT INTO task_assignees (task_id, profile_id, assigned_at)
--         SELECT id, assigned_to, updated_at
--         FROM tasks
--         WHERE assigned_to IS NOT NULL
--         ON CONFLICT (task_id, profile_id) DO NOTHING;
--
--         ALTER TABLE tasks DROP COLUMN assigned_to;
--     END IF;
-- END $$;
