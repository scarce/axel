-- Barrel App Supabase Schema
-- Run this in your Supabase SQL Editor: https://supabase.com/dashboard/project/YOUR_PROJECT/sql

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Todos table
CREATE TABLE IF NOT EXISTS todos (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT
);

-- Skills table
CREATE TABLE IF NOT EXISTS skills (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT
);

-- Contexts table
CREATE TABLE IF NOT EXISTS contexts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_todos_updated_at ON todos(updated_at);
CREATE INDEX IF NOT EXISTS idx_skills_updated_at ON skills(updated_at);
CREATE INDEX IF NOT EXISTS idx_contexts_updated_at ON contexts(updated_at);

-- Enable Row Level Security (optional but recommended)
ALTER TABLE todos ENABLE ROW LEVEL SECURITY;
ALTER TABLE skills ENABLE ROW LEVEL SECURITY;
ALTER TABLE contexts ENABLE ROW LEVEL SECURITY;

-- Allow all operations for now (you can restrict later with auth)
CREATE POLICY "Allow all for todos" ON todos FOR ALL USING (true);
CREATE POLICY "Allow all for skills" ON skills FOR ALL USING (true);
CREATE POLICY "Allow all for contexts" ON contexts FOR ALL USING (true);

-- Enable realtime for all tables
ALTER PUBLICATION supabase_realtime ADD TABLE todos;
ALTER PUBLICATION supabase_realtime ADD TABLE skills;
ALTER PUBLICATION supabase_realtime ADD TABLE contexts;
