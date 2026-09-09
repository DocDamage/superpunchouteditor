# Animation Editor

The Animation Editor allows you to create, modify, and preview fighter animations in Super Punch-Out!!.

## Understanding SNES Animations

### Animation Structure

Each animation consists of:
- **Frames**: Individual poses/sprites displayed
- **Timing**: How long each frame displays
- **Looping**: Whether animation repeats
- **Triggers**: Events that occur during playback

### Frame Components

Each animation frame contains:
- **Pose ID**: Which fighter pose to display
- **Duration**: Frame count (at 60fps)
- **Flags**: Flip, effects, etc.
- **Sound**: Audio trigger

### Timing

Super Punch-Out!! runs at 60 frames per second:
- Duration of 6 = 0.1 seconds
- Duration of 30 = 0.5 seconds
- Duration of 60 = 1 second

## The Animation Editor Interface

### Timeline View

The timeline displays:
- **Frame strips**: Visual representation of each frame
- **Duration bars**: Length shows frame duration
- **Playhead**: Current playback position
- **Key markers**: Special event points

### Frame List

Detailed frame information:
- Frame number and thumbnail
- Duration (in frames)
- Pose reference
- Effect flags
- Sound ID (if any)

### Preview Window

Real-time animation preview:
- Play/pause controls
- Speed adjustment (0.25x to 2x)
- Background selection
- Grid overlay toggle

## Creating Animations

### From Scratch

1. Click **"New Animation"**
2. Name your animation
3. Select initial fighter
4. Add frames using the **+** button
5. Set durations for each frame
6. Test playback

### From Existing Animation

1. Find the animation to duplicate
2. Click **"Duplicate"**
3. Give it a new name
4. Modify as needed
5. Save changes

### Using the Frame Library

Pre-made frame sequences:
1. Open **Frame Library**
2. Browse by category (punches, dodges, hits)
3. Select desired sequence
4. Click **"Insert"** at desired position

## Editing Frames

### Adding Frames

Methods to add frames:
- **Duplicate**: Copy selected frame
- **Insert**: Add blank frame
- **Import**: Load from file
- **Library**: Use preset frames

### Removing Frames

- **Delete**: Remove selected frame
- **Clear**: Remove all frames
- **Trim**: Remove before/after playhead

### Adjusting Timing

Change frame duration:
1. Select frame in timeline
2. Drag duration handle
3. Or enter value in properties panel

Duration guidelines:
- **Quick moves**: 2-6 frames
- **Normal punches**: 8-15 frames
- **Recovery**: 10-30 frames
- **Idle loops**: 30-60 frames

## Animation Categories

### Idle Animations

Fighter's default stance:
- Continuous loop
- Subtle movement
- Breathing effect

### Attack Animations

Punch sequences:
- **Windup**: Preparation frames
- **Strike**: Contact frame
- **Recovery**: Return to idle

### Defensive Animations

Dodge and block:
- Quick movements
- Low duration frames
- Return to idle

### Hit Reactions

Taking damage:
- Impact frame
- Recoil sequence
- Recovery options

### Special Moves

Unique attacks:
- Multi-part sequences
- Screen effects
- Sound triggers

## Advanced Features

### Animation Blending

Smooth transitions between animations:
1. Enable "Auto-blend" in settings
2. Set blend duration
3. Editor interpolates poses

### Event Triggers

Add events at specific frames:
- **Sound effects**: Punch sounds, vocals
- **Screen shake**: Impact effects
- **Flash effects**: White/red flashes
- **State changes**: Invulnerability, etc.

### Conditional Animation

Branch based on game state:
- Health threshold
- Round number
- Random chance
- Player input

## Preview Options

### Playback Controls

- **Play/Pause**: Spacebar
- **Step forward**: Right arrow
- **Step back**: Left arrow
- **Go to start**: Home
- **Go to end**: End

### View Settings

- **Background**: Ring, transparent, custom color
- **Opponent**: Show other fighter
- **Hitboxes**: Display collision data
- **Debug info**: Frame count, timing

### Comparison Mode

Compare two animations side-by-side:
1. Enable "Compare Mode"
2. Select second animation
3. Synchronized playback
4. Frame difference highlighting

## Import/Export

### JSON Format

Export animations as JSON for editing:
```json
{
  "name": "Custom Punch",
  "category": "PunchRight",
  "looping": false,
  "frames": [
    {"pose_id": 12, "duration": 8, "flags": 0},
    {"pose_id": 13, "duration": 6, "flags": 0},
    {"pose_id": 14, "duration": 12, "flags": 0}
  ]
}
```

### Sharing Animations

Share with community:
1. Export as JSON
2. Share file on forums/Discord
3. Others can import directly

### Batch Export

Export multiple animations:
1. Select animations in list
2. Choose "Batch Export"
3. Select destination folder
4. Files named automatically

## Validation and Testing

### Animation Validator

Automatic checks:
- **Empty frames**: No zero-duration frames
- **Missing poses**: All pose IDs valid
- **Extreme durations**: Warn on very long frames
- **Total length**: Very long animation warning

### In-Game Testing

Test in emulator:
1. Save changes
2. Export IPS patch
3. Apply to ROM
4. Test in emulator (F5 shortcut)

## Best Practices

1. **Reference original**: Study existing animations
2. **Keep timing snappy**: Avoid sluggish feeling
3. **Test at full speed**: Slow motion hides issues
4. **Consider hit frames**: Timing affects gameplay
5. **Save versions**: Duplicate before major changes

## Common Issues

### "Animation too fast/slow"
- Check duration values
- Verify 60fps assumption
- Test in-game

### "Pose not showing"
- Verify pose ID exists
- Check fighter selection
- Reload fighter data

### "Choppy playback"
- Reduce number of frames
- Check computer performance
- Lower preview quality

## Related Topics

- [Frame Reconstructor](./frame-reconstructor.md)
- [Sprite Editing](./sprite-editing.md)
- [Script Editing](./script-editing.md)
