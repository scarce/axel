# Axel Launch Video

A Remotion-powered launch video for Axel - AI Agent Orchestration for macOS.

## Development

```bash
# From the monorepo root
pnpm dev:video

# Or from this directory
pnpm dev
```

This will open Remotion Studio at http://localhost:3000

## Rendering

```bash
# Render to MP4
pnpm build

# Output will be in out/launch-video.mp4
```

## Scenes

The video consists of 6 scenes:

1. **IntroScene** - Animated logo with typewriter effect and starfield
2. **TaskQueueScene** - Showcases the task queue feature with screenshot
3. **SkillsScene** - Demonstrates portable skills with orbiting agents
4. **InboxScene** - Shows the approval inbox with animated cards
5. **HeroScene** - Full app hero screenshot with glow effects
6. **OutroScene** - CTA with download button and platform badges

## Duration

- Total: 15 seconds (450 frames at 30fps)
- Resolution: 1920x1080 (Full HD)

## Assets

Place screenshots and images in the `public/` folder:
- `tasks.png` - Task queue screenshot
- `hero.png` - Full app screenshot
