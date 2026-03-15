-- Fix ALL RLS Policies
-- The original policies used LEFT JOIN which fails when organization_id checks don't match
-- This uses nested EXISTS which is clearer and works correctly

-- ============================================
-- TASKS
-- ============================================
DROP POLICY IF EXISTS "Users can view tasks in their workspaces" ON public.tasks;
DROP POLICY IF EXISTS "Users can insert tasks in their workspaces" ON public.tasks;
DROP POLICY IF EXISTS "Users can update tasks in their workspaces" ON public.tasks;
DROP POLICY IF EXISTS "Users can delete tasks in their workspaces" ON public.tasks;

CREATE POLICY "Users can view tasks in their workspaces" ON public.tasks
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            WHERE w.id = tasks.workspace_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );

CREATE POLICY "Users can insert tasks in their workspaces" ON public.tasks
    FOR INSERT WITH CHECK (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            WHERE w.id = workspace_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );

CREATE POLICY "Users can update tasks in their workspaces" ON public.tasks
    FOR UPDATE USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            WHERE w.id = tasks.workspace_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );

CREATE POLICY "Users can delete tasks in their workspaces" ON public.tasks
    FOR DELETE USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            WHERE w.id = tasks.workspace_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );

-- ============================================
-- SKILLS
-- ============================================
DROP POLICY IF EXISTS "Users can manage skills in their workspaces" ON public.skills;

CREATE POLICY "Users can manage skills in their workspaces" ON public.skills
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            WHERE w.id = skills.workspace_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );

-- ============================================
-- CONTEXTS
-- ============================================
DROP POLICY IF EXISTS "Users can manage contexts in their workspaces" ON public.contexts;

CREATE POLICY "Users can manage contexts in their workspaces" ON public.contexts
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            WHERE w.id = contexts.workspace_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );

-- ============================================
-- TERMINALS
-- ============================================
DROP POLICY IF EXISTS "Users can manage terminals in their workspaces" ON public.terminals;

CREATE POLICY "Users can manage terminals in their workspaces" ON public.terminals
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            WHERE w.id = terminals.workspace_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );

-- ============================================
-- HINTS (linked through tasks)
-- ============================================
DROP POLICY IF EXISTS "Users can manage hints in their workspaces" ON public.hints;

CREATE POLICY "Users can manage hints in their workspaces" ON public.hints
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.tasks t
            JOIN public.workspaces w ON w.id = t.workspace_id
            WHERE t.id = hints.task_id
            AND (
                w.owner_id = auth.uid()
                OR EXISTS (
                    SELECT 1 FROM public.organization_members om
                    WHERE om.organization_id = w.organization_id
                    AND om.user_id = auth.uid()
                )
            )
        )
    );
