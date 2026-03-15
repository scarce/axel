-- Organization Invitations
-- Allows org admins/owners to invite new members by email

CREATE TABLE IF NOT EXISTS public.organization_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES public.organizations(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('admin', 'member')),
    invited_by UUID REFERENCES public.profiles(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'declined', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days'),
    UNIQUE(organization_id, email)
);

ALTER TABLE public.organization_invitations ENABLE ROW LEVEL SECURITY;

-- Org members can view invitations for their org
CREATE POLICY "Org members can view invitations" ON public.organization_invitations
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = organization_invitations.organization_id
            AND user_id = auth.uid()
        )
    );

-- Only org admins/owners can create invitations
CREATE POLICY "Org admins can create invitations" ON public.organization_invitations
    FOR INSERT WITH CHECK (
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = organization_invitations.organization_id
            AND user_id = auth.uid()
            AND role IN ('owner', 'admin')
        )
    );

-- Only org admins/owners can update invitations (e.g., cancel them)
CREATE POLICY "Org admins can update invitations" ON public.organization_invitations
    FOR UPDATE USING (
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = organization_invitations.organization_id
            AND user_id = auth.uid()
            AND role IN ('owner', 'admin')
        )
    );

-- Only org admins/owners can delete invitations
CREATE POLICY "Org admins can delete invitations" ON public.organization_invitations
    FOR DELETE USING (
        EXISTS (
            SELECT 1 FROM public.organization_members
            WHERE organization_id = organization_invitations.organization_id
            AND user_id = auth.uid()
            AND role IN ('owner', 'admin')
        )
    );

-- Index for efficient queries
CREATE INDEX IF NOT EXISTS idx_org_invitations_org ON public.organization_invitations(organization_id);
CREATE INDEX IF NOT EXISTS idx_org_invitations_email ON public.organization_invitations(email);
CREATE INDEX IF NOT EXISTS idx_org_invitations_status ON public.organization_invitations(status);

-- Enable realtime for invitations
ALTER PUBLICATION supabase_realtime ADD TABLE public.organization_invitations;
