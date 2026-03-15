-- Barrel App Schema
-- Run this FIRST, then run 001_auto_create_profile.sql

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================
-- PROFILES (linked to auth.users)
-- ============================================
CREATE TABLE IF NOT EXISTS public.profiles (
    id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
    email TEXT,
    full_name TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own profile" ON public.profiles
    FOR SELECT USING (auth.uid() = id);

CREATE POLICY "Users can update own profile" ON public.profiles
    FOR UPDATE USING (auth.uid() = id);

-- ============================================
-- ORGANIZATIONS
-- ============================================
CREATE TABLE IF NOT EXISTS public.organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE public.organizations ENABLE ROW LEVEL SECURITY;

-- ============================================
-- ORGANIZATION MEMBERS
-- ============================================
CREATE TABLE IF NOT EXISTS public.organization_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES public.organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES public.profiles(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'admin', 'member')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, user_id)
);

ALTER TABLE public.organization_members ENABLE ROW LEVEL SECURITY;

-- Users can see orgs they belong to
CREATE POLICY "Users can view own org memberships" ON public.organization_members
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can view orgs they belong to" ON public.organizations
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = organizations.id
            AND user_id = auth.uid()
        )
    );

-- ============================================
-- WORKSPACES
-- ============================================
CREATE TABLE IF NOT EXISTS public.workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    owner_id UUID REFERENCES public.profiles(id) ON DELETE SET NULL,
    organization_id UUID REFERENCES public.organizations(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE public.workspaces ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view workspaces in their orgs" ON public.workspaces
    FOR SELECT USING (
        owner_id = auth.uid() OR
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = workspaces.organization_id
            AND user_id = auth.uid()
        )
    );

CREATE POLICY "Users can insert workspaces in their orgs" ON public.workspaces
    FOR INSERT WITH CHECK (
        owner_id = auth.uid() OR
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = workspaces.organization_id
            AND user_id = auth.uid()
            AND role IN ('owner', 'admin')
        )
    );

CREATE POLICY "Users can update workspaces they own" ON public.workspaces
    FOR UPDATE USING (
        owner_id = auth.uid() OR
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = workspaces.organization_id
            AND user_id = auth.uid()
            AND role IN ('owner', 'admin')
        )
    );

-- ============================================
-- TASKS
-- ============================================
CREATE TABLE IF NOT EXISTS public.tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID REFERENCES public.workspaces(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'completed', 'in_review', 'aborted')),
    priority INT NOT NULL DEFAULT 0,
    created_by UUID REFERENCES public.profiles(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

ALTER TABLE public.tasks ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view tasks in their workspaces" ON public.tasks
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            LEFT JOIN public.organization_members om ON om.organization_id = w.organization_id
            WHERE w.id = tasks.workspace_id
            AND (w.owner_id = auth.uid() OR om.user_id = auth.uid())
        )
    );

CREATE POLICY "Users can insert tasks in their workspaces" ON public.tasks
    FOR INSERT WITH CHECK (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            LEFT JOIN public.organization_members om ON om.organization_id = w.organization_id
            WHERE w.id = workspace_id
            AND (w.owner_id = auth.uid() OR om.user_id = auth.uid())
        )
    );

CREATE POLICY "Users can update tasks in their workspaces" ON public.tasks
    FOR UPDATE USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            LEFT JOIN public.organization_members om ON om.organization_id = w.organization_id
            WHERE w.id = tasks.workspace_id
            AND (w.owner_id = auth.uid() OR om.user_id = auth.uid())
        )
    );

-- ============================================
-- SKILLS
-- ============================================
CREATE TABLE IF NOT EXISTS public.skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID REFERENCES public.workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE public.skills ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can manage skills in their workspaces" ON public.skills
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            LEFT JOIN public.organization_members om ON om.organization_id = w.organization_id
            WHERE w.id = skills.workspace_id
            AND (w.owner_id = auth.uid() OR om.user_id = auth.uid())
        )
    );

-- ============================================
-- CONTEXTS
-- ============================================
CREATE TABLE IF NOT EXISTS public.contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID REFERENCES public.workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE public.contexts ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can manage contexts in their workspaces" ON public.contexts
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            LEFT JOIN public.organization_members om ON om.organization_id = w.organization_id
            WHERE w.id = contexts.workspace_id
            AND (w.owner_id = auth.uid() OR om.user_id = auth.uid())
        )
    );

-- ============================================
-- TERMINALS
-- ============================================
CREATE TABLE IF NOT EXISTS public.terminals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID REFERENCES public.workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'idle' CHECK (status IN ('idle', 'running', 'stopped')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE public.terminals ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can manage terminals in their workspaces" ON public.terminals
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.workspaces w
            LEFT JOIN public.organization_members om ON om.organization_id = w.organization_id
            WHERE w.id = terminals.workspace_id
            AND (w.owner_id = auth.uid() OR om.user_id = auth.uid())
        )
    );

-- ============================================
-- HINTS
-- ============================================
CREATE TABLE IF NOT EXISTS public.hints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    terminal_id UUID REFERENCES public.terminals(id) ON DELETE CASCADE,
    task_id UUID REFERENCES public.tasks(id) ON DELETE CASCADE,
    type TEXT NOT NULL DEFAULT 'text_input' CHECK (type IN ('exclusive_choice', 'multiple_choice', 'text_input')),
    title TEXT NOT NULL,
    description TEXT,
    options JSONB,
    response TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'answered')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    answered_at TIMESTAMPTZ
);

ALTER TABLE public.hints ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can manage hints in their workspaces" ON public.hints
    FOR ALL USING (
        EXISTS (
            SELECT 1 FROM public.tasks t
            JOIN public.workspaces w ON w.id = t.workspace_id
            LEFT JOIN public.organization_members om ON om.organization_id = w.organization_id
            WHERE t.id = hints.task_id
            AND (w.owner_id = auth.uid() OR om.user_id = auth.uid())
        )
    );

-- ============================================
-- INDEXES
-- ============================================
CREATE INDEX IF NOT EXISTS idx_org_members_user ON public.organization_members(user_id);
CREATE INDEX IF NOT EXISTS idx_org_members_org ON public.organization_members(organization_id);
CREATE INDEX IF NOT EXISTS idx_workspaces_owner ON public.workspaces(owner_id);
CREATE INDEX IF NOT EXISTS idx_workspaces_org ON public.workspaces(organization_id);
CREATE INDEX IF NOT EXISTS idx_tasks_workspace ON public.tasks(workspace_id);
CREATE INDEX IF NOT EXISTS idx_tasks_updated ON public.tasks(updated_at);
CREATE INDEX IF NOT EXISTS idx_skills_workspace ON public.skills(workspace_id);
CREATE INDEX IF NOT EXISTS idx_contexts_workspace ON public.contexts(workspace_id);
CREATE INDEX IF NOT EXISTS idx_terminals_workspace ON public.terminals(workspace_id);
CREATE INDEX IF NOT EXISTS idx_hints_task ON public.hints(task_id);

-- ============================================
-- REALTIME
-- ============================================
ALTER PUBLICATION supabase_realtime ADD TABLE public.tasks;
ALTER PUBLICATION supabase_realtime ADD TABLE public.skills;
ALTER PUBLICATION supabase_realtime ADD TABLE public.contexts;
ALTER PUBLICATION supabase_realtime ADD TABLE public.hints;
ALTER PUBLICATION supabase_realtime ADD TABLE public.workspaces;
