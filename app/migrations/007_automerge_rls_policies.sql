-- Migration: Update RLS policies to include automerge_doc column
-- This ensures that authenticated users can read and write automerge_doc for their entities

-- Note: If your existing policies use column-level security, you may need to recreate them.
-- The automerge_doc column should be accessible whenever the row itself is accessible.

-- For tasks table - ensure the policy allows all columns including automerge_doc
-- Drop and recreate policies if they use specific column lists

-- Check if policies need to be updated by examining existing policies:
-- SELECT * FROM pg_policies WHERE tablename = 'tasks';

-- If your policies look like:
--   CREATE POLICY "users can update own tasks" ON tasks
--   FOR UPDATE USING (auth.uid() = created_by)
--   WITH CHECK (auth.uid() = created_by);
--
-- Then automerge_doc should already be included (policies apply to all columns by default)

-- However, if you have column-level restrictions, add these grants:

-- Grant column-level access for authenticated users (if using column security)
-- GRANT SELECT, UPDATE (automerge_doc) ON tasks TO authenticated;
-- GRANT SELECT, UPDATE (automerge_doc) ON workspaces TO authenticated;
-- GRANT SELECT, UPDATE (automerge_doc) ON skills TO authenticated;
-- GRANT SELECT, UPDATE (automerge_doc) ON contexts TO authenticated;
-- GRANT SELECT, UPDATE (automerge_doc) ON terminals TO authenticated;
-- GRANT SELECT, UPDATE (automerge_doc) ON hints TO authenticated;
-- GRANT SELECT, UPDATE (automerge_doc) ON organizations TO authenticated;

-- More likely, ensure your UPDATE policies don't restrict columns.
-- Standard RLS policies that should work:

-- Example: If you need to recreate the tasks update policy to include all columns:
-- DROP POLICY IF EXISTS "users can update tasks in their workspaces" ON tasks;
-- CREATE POLICY "users can update tasks in their workspaces" ON tasks
--   FOR UPDATE
--   USING (
--     workspace_id IN (
--       SELECT w.id FROM workspaces w
--       JOIN organization_members om ON w.organization_id = om.organization_id
--       WHERE om.user_id = auth.uid()
--     )
--   )
--   WITH CHECK (
--     workspace_id IN (
--       SELECT w.id FROM workspaces w
--       JOIN organization_members om ON w.organization_id = om.organization_id
--       WHERE om.user_id = auth.uid()
--     )
--   );

-- Verify the automerge_doc column exists and is accessible:
DO $$
BEGIN
    -- Check if columns exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tasks' AND column_name = 'automerge_doc') THEN
        RAISE EXCEPTION 'automerge_doc column missing from tasks table. Run migration 006 first.';
    END IF;
END $$;

-- If your issue is that Supabase client can't write to automerge_doc,
-- it's likely because:
-- 1. The column doesn't exist (run migration 006)
-- 2. RLS policy uses USING clause that doesn't match for UPDATE
-- 3. There's a trigger or constraint blocking the write

-- Debug query to check policies:
-- SELECT schemaname, tablename, policyname, permissive, roles, cmd, qual, with_check
-- FROM pg_policies
-- WHERE tablename IN ('tasks', 'workspaces', 'skills', 'contexts', 'terminals', 'hints', 'organizations');
