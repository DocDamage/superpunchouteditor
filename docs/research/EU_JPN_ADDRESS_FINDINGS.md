# EU and JPN ROM Table Address Findings

## Executive Summary

Analysis of European (EU) and Japanese (JPN) Super Punch-Out!! ROMs completed.
All three regions (USA, EU, JPN) share the **same base addresses** for most tables,
but data offsets within those tables vary. Palettes and icons are shifted by
consistent offsets in EU and JPN ROMs.

---

## Fighter Header Table

| Region | PC Address | SNES Address | Confidence | Notes |
|--------|------------|--------------|------------|-------|
| **USA** | 0x048000 | $89:8000 | HIGH | Reference baseline |
| **EU** | 0x048000 | $89:8000 | HIGH | Same location as USA |
| **JPN** | 0x048000 | $89:8000 | HIGH | Same location as USA |

### Header Entry Comparison

Each fighter entry is 32 bytes. The following shows similarity to USA:

| Fighter | USA→EU | USA→JPN | Status |
|---------|--------|---------|--------|
| Gabby Jay | 78.1% | 78.1% | DIFFERENT data values |
| Bear Hugger | 71.9% | 71.9% | DIFFERENT data values |
| Piston Hurricane | 71.9% | 71.9% | DIFFERENT data values |
| Bald Bull | 56.2% | 65.6% | DIFFERENT data values |
| **Bob Charlie** | **100%** | **100%** | **IDENTICAL** |
| Dragon Chan | 90.6% | 100% | Structure same, some values differ |
| Masked Muscle | 87.5% | 100% | Structure same, some values differ |
| Mr. Sandman | 90.6% | 100% | Structure same, some values differ |
| Aran Ryan | 87.5% | 100% | Structure same, some values differ |
| Heike Kagero | 84.4% | 100% | Structure same, some values differ |
| Mad Clown | 87.5% | 100% | Structure same, some values differ |
| Super Macho Man | 96.9% | 96.9% | Minor differences |
| Narcis Prince | 59.4% | 59.4% | More differences |
| Hoy Quarlow | 59.4% | 59.4% | More differences |
| Rick Bruiser | 50.0% | 50.0% | Significant differences |
| Nick Bruiser | 46.9% | 46.9% | Significant differences |

### Key Finding
**Bob Charlie's header is IDENTICAL across all regions** - can be used as an anchor point.

---

## Palette Data Tables

All palettes are shifted by consistent offsets from USA addresses.

### Palette Address Mapping

| Fighter | USA | EU | JPN |
|---------|-----|----|----|
| **Gabby Jay** | 0x06B9DA | **0x06B9D3** (-7) | **0x06B8CB** (-271) |
| **Bear Hugger** | 0x06BC3C | **0x06BC35** (-7) | **0x06BB2D** (-271) |
| **Piston Hurricane** | 0x06BE9E | **0x06BE97** (-7) | **0x06BD8F** (-271) |
| **Bald Bull** | 0x06C100 | **0x06C0F9** (-7) | **0x06BFF1** (-271) |
| **Bob Charlie** | 0x06C382 | **0x06C37B** (-7) | **0x06C273** (-271) |
| **Dragon Chan** | 0x06C5E4 | **0x06C5DD** (-7) | **0x06C4D5** (-271) |
| **Masked Muscle** | 0x06C846 | **0x06C83F** (-7) | **0x06C737** (-271) |
| **Mr. Sandman** | 0x06CA88 | **0x06CA81** (-7) | **0x06C979** (-271) |
| **Aran Ryan** | 0x06CCCA | **0x06CCC3** (-7) | **0x06CBBB** (-271) |
| **Heike Kagero** | 0x06CF2C | **0x06CF25** (-7) | **0x06CE1D** (-271) |
| **Mad Clown** | 0x06D18E | **0x06D187** (-7) | **0x06D07F** (-271) |
| **Super Macho Man** | 0x06D3F0 | **0x06D3E9** (-7) | **0x06D2E1** (-271) |
| **Narcis Prince** | 0x06D652 | **0x06D64B** (-7) | **0x06D543** (-271) |
| **Hoy Quarlow** | 0x06D8B4 | **0x06D8AD** (-7) | **0x06D7A5** (-271) |
| **Rick Bruiser** | 0x06DB16 | **0x06DB0F** (-7) | **0x06DA07** (-271) |
| **Nick Bruiser** | 0x06DD58 | **0x06DD51** (-7) | **0x06DC49** (-271) |

### Summary
- **EU Palettes**: Offset -7 bytes from USA
- **JPN Palettes**: Offset -271 bytes from USA
- **Confidence**: HIGH - All palettes found with exact pattern matches

---

## Icon/Small Portrait Tables

Icons follow the same offset pattern as palettes.

| Fighter | USA | EU | JPN |
|---------|-----|----|----|
| Gabby Jay | 0x06B7D8 | 0x06B7D1 (-7) | 0x06B6C9 (-271) |
| Bear Hugger | 0x06BA3A | 0x06BA33 (-7) | 0x06B92B (-271) |
| Piston Hurricane | 0x06BC9C | 0x06BC95 (-7) | 0x06BB8D (-271) |
| Bald Bull | 0x06BEFE | 0x06BEF7 (-7) | 0x06BDEF (-271) |

---

## Circuit Assignment Table

| Region | PC Address | SNES Address | Confidence |
|--------|------------|--------------|------------|
| **USA** | 0x06ABD4 | $8D:ABD4 | MEDIUM |
| **EU** | 0x06ABCD | $8D:ABCD | MEDIUM |
| **JPN** | 0x06ABE8 | $8D:ABE8 | MEDIUM |

Note: Found by searching for Minor Circuit pattern (bytes: 00 01 02 03)

---

## Header Pointer Differences

Pointers within fighter headers that differ from USA:

### EU ROM Pointer Changes
- Many pose_table_ptr, ai_script_ptr, and corner_man_ptr values differ
- Some fighters (Bob Charlie, Dragon Chan, Masked Muscle, Super Macho Man) have identical pointers

### JPN ROM Pointer Changes
- Similar pattern to EU but with different offset values
- Bob Charlie, Dragon Chan, Masked Muscle, Super Macho Man have identical pointers to USA

---

## Japanese Text Regions

JPN ROM contains Japanese text in different encoding. Potential text areas found:

| Address Range | Description |
|---------------|-------------|
| 0x060200-0x061000 | Japanese font/tile data |
| 0x061F00-0x062500 | Dialog text area |
| 0x067200-0x068000 | Character names/messages |

---

## Regions Identical Across All Versions

The following regions are **byte-for-byte identical** in USA, EU, and JPN:

| Region | Description |
|--------|-------------|
| Portrait Graphics (Bank $10) | Large portraits are identical |
| Compressed Sprites (Banks $30-$3C) | Sprite data identical |

---

## Recommended Implementation Strategy

### For Multi-ROM Support:

1. **Use Base Addresses**: Fighter header table at 0x048000 for all regions
2. **Apply Offsets**: Use region-specific offset tables for palettes/icons
3. **Pointer Fixups**: Adjust header pointers based on region

### Offset Tables:

```rust
// Pseudo-code for region offsets
const PALETTE_OFFSETS: &[(Region, i32)] = &[
    (Region::USA, 0),
    (Region::EU, -7),
    (Region::JPN, -271),
];

const ICON_OFFSETS: &[(Region, i32)] = &[
    (Region::USA, 0),
    (Region::EU, -7),
    (Region::JPN, -271),
];
```

---

## Confidence Levels Summary

| Table Type | USA | EU | JPN |
|------------|-----|----|----|
| Fighter Header | HIGH | HIGH | HIGH |
| Palettes | HIGH | HIGH | HIGH |
| Icons | HIGH | HIGH | HIGH |
| Circuit Table | MEDIUM | MEDIUM | MEDIUM |
| AI Scripts | LOW | LOW | LOW |
| Text Tables | N/A | LOW | LOW |

---

## Files Generated

- `eu_jpn_table_addresses.json` - Detailed JSON with all findings
- `region_analysis_findings.json` - Summary of key findings
- `rom_pattern_search_results.json` - Raw pattern search results

---

*Analysis completed: 2026-03-20*
*Tools: Python 3 with custom ROM analysis scripts*
