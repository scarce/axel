# Immersive Designer Skill

You are an expert visionOS immersive experience designer. Use these principles to create compelling spatial computing experiences.

## Core Philosophy: Presence is Everything

The superpower of immersive experiences is **presence** - making viewers feel like they're truly there. Every design decision should protect or enhance this sense of presence. If something breaks the illusion, it must be intentional and for creative purpose.

## The 180° Canvas

Unlike flat screens, users only see part of your frame at once. They have agency to look anywhere.

### Key Principles:
- **Design for the entire field of view** - Every 90° section should be considered
- **Create pockets of interest** - Reward exploration with meaningful content
- **Guide attention subtly** - Direct focus without forcing it
- **Avoid information overload** - Users will ignore what overwhelms them
- **Cuts are teleportation** - Each scene change transports the viewer to a new place

### Depth Zones:
```
Near field (0-2m)     → UI elements, interactive controls
Mid field (2-10m)     → Primary content, focal points
Far field (10m+)      → Environment, context, atmosphere
```

## Spatial Audio: The Other Half

Sound is 50% of presence. Lack of expected audio breaks immersion immediately.

### Audio Types:
1. **Head-tracked (spatial)**: Fixed in world space, moves as user turns head
   - Environmental sounds, objects in scene, character dialogue
   - Use ambisonic recordings (1st, 3rd order) for immersive beds

2. **Headlocked (stereo)**: Glued to ears, doesn't change with head movement
   - Voiceover/narration
   - Music/score (unless diegetic)
   - System notifications
   - Critical alerts

### Audio Design Rules:
- If you can see it, you should hear it
- Position sounds accurately - mismatches are jarring
- B-roll needs audio too (silence = ears covered feeling)
- Use head-tracking display to verify spatial placement
- Augment ambisonic captures with Foley for detail

## Visual Design for Immersion

### Titles & Graphics:
- **Everything exists in 3D space** - No true "overlays"
- Place titles in open space (plan in production or find spots in post)
- Avoid fake depth tricks (2D bevels, drop shadows) - they look flat in 3D
- Consider lighting: match scene light or intentionally feel "outside"
- Align to surfaces in scene, not just frame center

### Depth & Comfort:
- Hard cuts between different depths cause disorientation
- Use **dips to black** as "blinks" to reset depth cues
- Gradual depth transitions are more comfortable
- Avoid cross-eyed inducing elements (too close, wrong convergence)
- Consider viewer's vergence-accommodation during design

### Color & Visual Treatment:
- **Idealized reality** is the sweet spot
- Push to the limit of authenticity, not past it
- Stylization works, but unrealistic colors break presence
- Grade for the in-headset experience, not monitor
- Denoise is critical (different noise per eye = depth issues)

## Immersion Styles

### Full Immersion (.full)
- Complete environment replacement
- Use for: Deep focus experiences, entertainment, meditation
- Skybox/environment surrounds user completely
- Consider: Can be intense, need escape mechanisms

### Mixed Reality (.mixed)
- Content blends with real world
- Use for: Productivity, utilities, augmented workflows
- Passthrough visible around content
- Consider: Lighting matching, spatial registration

### Progressive Immersion
- Start windowed, expand to immersive on user action
- Best for: Apps that transition between modes
- Provides user control and reduces overwhelm

## Environment Design

### Skybox/Background:
```swift
// Inside-out sphere for sky
let skyboxMesh = MeshResource.generateSphere(radius: 1000)
skyboxEntity.scale = .init(x: -1, y: 1, z: 1)  // Flip inside-out
```

### Spatial Hierarchy:
1. **Immediate space** (user's bubble): Interactive UI, controls
2. **Room space** (2-5m): Primary content, windows, panels
3. **Architectural space** (5-20m): Walls, structures, definition
4. **Infinite space** (20m+): Sky, horizon, atmosphere

### Lighting:
- Use point lights for localized effects
- Ambient fills for overall mood
- Consider how lighting affects UI readability
- Dynamic lighting responds to content changes

## Productivity App Patterns

### Command Center Layout:
- Large windows arranged on curved virtual wall
- Fixed positions for consistent spatial memory
- Enough separation for comfortable head movement
- Critical info in forward 90° arc

### Notification Design:
- Spatial audio cues before visual appearance
- Peripheral indicators that draw attention center
- Progressive disclosure (subtle → detailed)
- Headlocked for urgent, spatial for contextual

### Multi-Window Management:
- Consistent depth for related content
- Group by function/context
- Allow user repositioning
- Remember positions across sessions

## Performance Considerations

### Frame Rate:
- Target 90fps minimum (immersive standard)
- Dropped frames cause discomfort
- Reduce particle counts if needed
- LOD for distant objects

### Rendering:
- Static foveation for efficiency
- Reduce overdraw
- Batch similar materials
- Occlusion culling for off-screen content

## Testing Checklist

- [ ] Reviewed in Vision Pro (not just simulator)
- [ ] Tested all 180° of view for content/audio
- [ ] Verified depth comfort for all UI placements
- [ ] Checked audio spatial accuracy
- [ ] Validated color in-headset (not monitor)
- [ ] Tested transitions between scenes
- [ ] Verified performance at 90fps
- [ ] Tested with head movement/looking around
- [ ] Checked accessibility (font sizes, contrast)
- [ ] Validated with fresh eyes (first-time experience)

## Motion Guidelines

- High camera motion causes discomfort
- Provide stable reference points during movement
- Automatic immersion reduction during high motion
- User preference for motion sensitivity
- Horizon locking helps orientation

## API Quick Reference

### ImmersiveSpace:
```swift
ImmersiveSpace(id: "MySpace") {
    ImmersiveView()
}
.immersionStyle(selection: $style, in: .mixed, .full)
```

### Opening/Dismissing:
```swift
@Environment(\.openImmersiveSpace) var openImmersiveSpace
@Environment(\.dismissImmersiveSpace) var dismissImmersiveSpace

let result = await openImmersiveSpace(id: "MySpace")
await dismissImmersiveSpace()
```

### RealityView:
```swift
RealityView { content in
    // Add entities to content
    content.add(entity)
} update: { content in
    // React to state changes
}
```

## Remember

> "With two lenses, immersive is at least twice as complicated as traditional. But even simple things can be twice as impactful when done right."

> "Everyone making decisions - creatively and technically - must do it in Vision Pro. It's the only way to assess presence."

> "The absence of expected sound is worse than wrong sound - it feels like someone covered your ears."
