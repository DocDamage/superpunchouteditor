# Script Editing

Script editing allows you to modify fighter behavior, statistics, and AI patterns in Super Punch-Out!!.

## Understanding Fighter Scripts

### Script Structure

Fighter data is stored in specialized scripts:
- **Header**: Fighter statistics and properties
- **Move lists**: Available attacks and defenses
- **AI patterns**: Behavioral logic
- **Animation pointers**: Which animations to use

### Header Data

The 32-byte fighter header contains:
```
Offset 0-1: Health/stamina
Offset 2: Attack power
Offset 3: Defense rating
Offset 4: Speed
Offset 5-7: Special flags
Offset 8-15: Name/ID
Offset 16-31: Additional stats
```

## The Script Viewer

### Fighter List

Browse all fighters:
- **Sort by**: Name, circuit, difficulty
- **Filter**: Circuit, weight class
- **Search**: Find by name
- **Favorites**: Star frequently edited fighters

### Header Editor

Edit fighter statistics:
- **Numeric inputs**: Direct value entry
- **Sliders**: Visual adjustment
- **Presets**: Load balanced stat sets
- **Validation**: Warn on extreme values

### Hex Viewer

Raw hex editor for advanced users:
- **Color coding**: Different data types
- **Navigation**: Jump to offsets
- **Search**: Find hex patterns
- **Bookmark**: Save important locations

## Fighter Statistics

### Basic Stats

| Stat | Range | Effect |
|------|-------|--------|
| Health | 1-255 | Hit points |
| Attack | 1-255 | Damage dealt |
| Defense | 1-255 | Damage reduction |
| Speed | 1-255 | Movement/agility |
| Stamina | 1-255 | Energy for moves |

### Special Stats

Additional properties:
- **Recovery**: Speed of getting up
- **Pattern complexity**: AI difficulty
- **Special move chance**: Frequency of signature moves
- **Taunt frequency**: How often they taunt

### Stat Relationships

Balanced fighters should have:
- **Trade-offs**: High attack → lower defense
- **Archetypes**: Tank (high HP), Speedster (fast), etc.
- **Circuit appropriate**: Minor Circuit = lower stats

## Editing Fighter Data

### Using the Form Editor

User-friendly interface:
1. Select fighter from list
2. Click **"Edit Stats"**
3. Modify values in form
4. Preview changes
5. Save to pending writes

### Validation

Automatic checks:
- **Range validation**: Values within valid range
- **Balance warnings**: Extreme stat combinations
- **Consistency**: Related stats make sense
- **Original comparison**: Highlight significant changes

### Reverting Changes

Restore original values:
- **Single stat**: Click reset button next to field
- **All stats**: Click **"Reset All"**
- **From backup**: Load saved fighter data

## AI Behavior Editing

### Pattern Structure

AI patterns consist of:
- **Triggers**: When to activate (health %, round, etc.)
- **Actions**: What to do when triggered
- **Probabilities**: Chance-based decisions
- **Counters**: Response to player actions

### Pattern Editor

Visual pattern editing:
1. Select AI tab
2. Current patterns displayed as flowchart
3. Click nodes to edit
4. Add/remove connections
5. Adjust probabilities

### Common Patterns

| Pattern | Description |
|---------|-------------|
| Aggressive | Attacks frequently |
| Defensive | Dodges and blocks often |
| Counter | Waits for player mistakes |
| Random | Unpredictable behavior |
| Scripted | Fixed attack sequences |

## Move Lists

### Available Moves

Configure which moves a fighter can use:
- **Jab**: Quick, low damage
- **Hook**: Medium speed/damage
- **Uppercut**: Slow, high damage
- **Special**: Unique signature moves

### Move Properties

Each move has:
- **Damage**: Base damage value
- **Speed**: Startup/recovery frames
- **Range**: Hitbox size/position
- **Energy cost**: Stamina required

### Disabling Moves

Remove moves from AI:
1. Open move list
2. Uncheck unwanted moves
3. AI won't use unchecked moves
4. Can still be used by player (if applicable)

## Advanced Features

### Conditional Logic

Script conditions:
```
IF health < 50% THEN
  increase_defense = true
  enable_desperation_moves = true
END IF
```

### Round-Based Changes

Different behavior per round:
- **Round 1**: Cautious, feeling out
- **Round 2**: More aggressive
- **Round 3**: Desperate, all-out

### Player Response

React to player patterns:
- **Counter frequency**: How often to counter
- **Adaptation**: Learn player tendencies
- **Punishment**: Capitalize on mistakes

## Testing Scripts

### In-Editor Testing

Simulate fights:
1. Click **"Test Fight"**
2. Configure opponent
3. Run simulation
4. View results/statistics

### Emulator Testing

Test in actual game:
1. Save changes
2. Export IPS patch
3. Apply to ROM
4. Play in emulator (F5)

### Debug Mode

Enable debug information:
- **Show AI state**: Current decision process
- **Display triggers**: When patterns activate
- **Log actions**: Record of AI decisions

## Import/Export

### JSON Export

Export as structured data:
```json
{
  "fighter_id": 5,
  "name": "Gabby Jay",
  "stats": {
    "health": 120,
    "attack": 45,
    "defense": 30,
    "speed": 60
  },
  "ai_patterns": [...]
}
```

### Batch Editing

Edit multiple fighters:
1. Select fighters in list
2. Choose "Batch Edit"
3. Apply stat multipliers
4. Mass save changes

## Best Practices

### Balance

1. **Test thoroughly**: One change affects many things
2. **Compare to originals**: Maintain game feel
3. **Consider circuits**: Match difficulty to circuit
4. **Document changes**: Track what you modified

### Safety

1. **Make backups**: Before major edits
2. **Test incrementally**: Small changes at a time
3. **Use version control**: Projects track changes
4. **Verify checksums**: Ensure ROM integrity

## Common Issues

### "Stats not saving"
- Click "Apply" after changes
- Check pending writes
- Verify ROM is loaded

### "AI behaves strangely"
- Check pattern probabilities sum to 100%
- Verify triggers are valid
- Test with debug mode

### "Fighter too easy/hard"
- Review stat balance
- Check AI pattern complexity
- Compare to similar fighters

## Related Topics

- [Getting Started](./getting-started.md)
- [Troubleshooting](./troubleshooting.md)
- [Patch Export](./patch-export.md)
