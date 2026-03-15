-- First, drop the existing trigger if it exists (to fix the error state)
DROP TRIGGER IF EXISTS on_auth_user_created ON auth.users;
DROP FUNCTION IF EXISTS public.handle_new_user();

-- Trigger function to create profile, personal org, and membership on signup
-- Note: Workspaces are created by the macOS app from directories, not auto-created here
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    new_org_id UUID;
    user_name TEXT;
    user_avatar TEXT;
BEGIN
    -- Extract metadata from GitHub OAuth
    user_name := COALESCE(
        NEW.raw_user_meta_data->>'user_name',
        NEW.raw_user_meta_data->>'name',
        NEW.raw_user_meta_data->>'full_name',
        split_part(NEW.email, '@', 1)
    );
    user_avatar := NEW.raw_user_meta_data->>'avatar_url';

    -- 1. Create the profile
    INSERT INTO public.profiles (id, email, full_name, avatar_url, created_at, updated_at)
    VALUES (
        NEW.id,
        NEW.email,
        user_name,
        user_avatar,
        NOW(),
        NOW()
    )
    ON CONFLICT (id) DO UPDATE SET
        email = EXCLUDED.email,
        full_name = EXCLUDED.full_name,
        avatar_url = EXCLUDED.avatar_url,
        updated_at = NOW();

    -- 2. Create personal organization
    new_org_id := gen_random_uuid();

    INSERT INTO public.organizations (id, name, slug, created_at, updated_at)
    VALUES (
        new_org_id,
        user_name || '''s Workspace',
        LOWER(REGEXP_REPLACE(user_name, '[^a-zA-Z0-9]', '-', 'g')) || '-' || SUBSTRING(new_org_id::text, 1, 8),
        NOW(),
        NOW()
    );

    -- 3. Add user as owner of their personal org
    INSERT INTO public.organization_members (id, organization_id, user_id, role, created_at)
    VALUES (
        gen_random_uuid(),
        new_org_id,
        NEW.id,
        'owner',
        NOW()
    );

    RETURN NEW;
EXCEPTION
    WHEN OTHERS THEN
        -- Log the error but don't fail the signup
        RAISE WARNING 'handle_new_user failed: %', SQLERRM;
        RETURN NEW;
END;
$$;

-- Create the trigger on auth.users
CREATE TRIGGER on_auth_user_created
    AFTER INSERT ON auth.users
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_new_user();

-- Grant necessary permissions
GRANT USAGE ON SCHEMA public TO supabase_auth_admin;
GRANT ALL ON public.profiles TO supabase_auth_admin;
GRANT ALL ON public.organizations TO supabase_auth_admin;
GRANT ALL ON public.organization_members TO supabase_auth_admin;
GRANT ALL ON public.workspaces TO supabase_auth_admin;
