-- Migration: Fix tasks RLS policies to allow inserts without workspace
-- The previous policy was too restrictive, requiring workspace access

-- Drop existing policies on tasks table
DROP POLICY IF EXISTS "Users can view their own tasks" ON tasks;
DROP POLICY IF EXISTS "Users can view tasks in their workspaces" ON tasks;
DROP POLICY IF EXISTS "Users can insert their own tasks" ON tasks;
DROP POLICY IF EXISTS "Users can insert tasks" ON tasks;
DROP POLICY IF EXISTS "Users can update their own tasks" ON tasks;
DROP POLICY IF EXISTS "Users can update tasks" ON tasks;
DROP POLICY IF EXISTS "Users can delete their own tasks" ON tasks;
DROP POLICY IF EXISTS "Users can delete tasks" ON tasks;

-- SELECT: Users can view tasks they created OR tasks in workspaces they have access to
CREATE POLICY "Users can view tasks"
    ON tasks FOR SELECT
    USING (
        created_by = auth.uid()
        OR EXISTS (
            SELECT 1 FROM task_assignees ta
            WHERE ta.task_id = tasks.id AND ta.profile_id = auth.uid()
        )
        OR (
            workspace_id IS NOT NULL AND EXISTS (
                SELECT 1 FROM workspaces w
                WHERE w.id = tasks.workspace_id
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
    );

-- INSERT: Users can create tasks if they set themselves as creator
-- workspace_id can be NULL (personal tasks) or a workspace they have access to
CREATE POLICY "Users can insert tasks"
    ON tasks FOR INSERT
    WITH CHECK (
        created_by = auth.uid()
        AND (
            workspace_id IS NULL
            OR EXISTS (
                SELECT 1 FROM workspaces w
                WHERE w.id = tasks.workspace_id
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
    );

-- UPDATE: Users can update tasks they created or are assigned to
CREATE POLICY "Users can update tasks"
    ON tasks FOR UPDATE
    USING (
        created_by = auth.uid()
        OR EXISTS (
            SELECT 1 FROM task_assignees ta
            WHERE ta.task_id = tasks.id AND ta.profile_id = auth.uid()
        )
    )
    WITH CHECK (
        created_by = auth.uid()
        OR EXISTS (
            SELECT 1 FROM task_assignees ta
            WHERE ta.task_id = tasks.id AND ta.profile_id = auth.uid()
        )
    );

-- DELETE: Users can delete tasks they created
CREATE POLICY "Users can delete tasks"
    ON tasks FOR DELETE
    USING (
        created_by = auth.uid()
    );
