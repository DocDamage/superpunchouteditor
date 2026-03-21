# Super Punch-Out!! ROM Structural Comparison Report

## Executive Summary

| Property | USA | Europe | Japan (NP) |
|----------|-----|--------|------------|
| **File Size** | 2,097,152 bytes (2MB) | 2,097,152 bytes (2MB) | 2,097,152 bytes (2MB) |
| **MD5 Hash** | 97FE7D7D2A1017F8480E60A365A373F0 | 49D619C689C21A6E41E2B98C31F5838A | BBB741D5844BD1C29531F42967369D9A |
| **Region Code** | 0x01 (North America) | 0x02 (Europe) | 0x00 (Japan) |
| **Version** | 1.51 | 1.51 | 1.51 |

## SNES Header Information

All three ROMs use **LoROM (Mode 20)** mapping:
- Header location: `0x7FB0`
- Maker code: `01` (Nintendo)
- Game code: `4Q  `
- Title: "Super Punch-Out!!"
- ROM size: 2MB (2^11)
- SRAM: 8KB
- Map mode: 0x20 (LoROM)

## Overall Similarity

| Comparison | Identical Bytes | Different Bytes | Similarity |
|------------|-----------------|-----------------|------------|
| USA vs Europe | 1,655,291 (78.93%) | 441,861 (21.07%) | 78.93% |
| USA vs Japan | 1,718,907 (81.96%) | 378,245 (18.04%) | 81.96% |
| Europe vs Japan | 1,699,826 (81.05%) | 397,326 (18.95%) | 81.05% |
| **All Three (Common)** | **1,637,048 (78.06%)** | - | **78.06%** |

## Key Finding: No Data Shifts

**IMPORTANT:** No significant data shifts were detected between regional versions. The ROMs share the same structure with only content differences in specific regions.

This means:
- ✅ Pointers/offsets should be compatible across versions
- ✅ Data locations are consistent
- ✅ Editor can likely work with all versions using same offsets

## Region-by-Region Analysis

### First 512KB (0x000000 - 0x07FFFF)
**Most divergent region** - Contains primary code and regionalized data:

| Bank Range | USA vs EU | USA vs JP | EU vs JP |
|------------|-----------|-----------|----------|
| 0-256KB (Bank 0-1) | 63.4% diff | 56.6% diff | 49.5% diff |
| 256-512KB (Bank 2-3) | 68.0% diff | 59.8% diff | 66.5% diff |

### 512KB - 1MB (0x080000 - 0x0FFFFF)
**Highly conserved** - Contains shared game assets:

| Bank Range | USA vs EU | USA vs JP | EU vs JP |
|------------|-----------|-----------|----------|
| 512-768KB (Bank 4-5) | 8.2% diff | **0.0% identical** | 8.2% diff |
| 768KB-1MB (Bank 6-7) | 0.1% diff | **0.0% identical** | 0.1% diff |

**Key observation:** Banks 4-7 are **identical between USA and Japan** versions! Europe differs slightly.

### Second Megabyte (0x100000 - 0x1FFFFF)
**Moderate divergence** - Contains additional game data:

| Bank Range | USA vs EU | USA vs JP | EU vs JP |
|------------|-----------|-----------|----------|
| 1-1.25MB (Bank 8-9) | 5.5% diff | 7.1% diff | 7.9% diff |
| 1.25-1.5MB (Bank 10-11) | 8.0% diff | 7.5% diff | 5.0% diff |
| 1.5-1.75MB (Bank 12-13) | 8.2% diff | 8.2% diff | 7.1% diff |
| 1.75-2MB (Bank 14-15) | 7.2% diff | 5.0% diff | 7.2% diff |

## Specific Difference Ranges

### Major Difference Blocks (USA vs Europe)

| Offset Range | Size | Description |
|--------------|------|-------------|
| 0x0007AA - 0x000925 | 380 bytes | Early header/data area |
| 0x000DE8 - 0x007613 | 26,668 bytes | Large code/data block |
| 0x0087BC - 0x00EBFF | 25,668 bytes | Text/data region |
| 0x010F5A - 0x017FFF | 28,838 bytes | Large data region |
| 0x040000 - 0x0459FF | 23,040 bytes | Code bank 2 |
| 0x048000 - 0x06FFFF | 163,840 bytes | **Massive difference block** |
| 0x087979 - 0x087AFF | 391 bytes | Small data patch |

### USA-Unique Regions (vs both EU and JP)

| Offset Range | Size | Notes |
|--------------|------|-------|
| 0x000DE8 - 0x007613 | 26,668 bytes | Large code/data block |
| 0x0089AF - 0x0093F7 | 2,633 bytes | Text region |
| 0x00957D - 0x00AECC | 6,480 bytes | Data table |
| 0x00B328 - 0x00E0F5 | 11,726 bytes | Large content block |

### Europe-Unique Regions

| Offset Range | Size | Notes |
|--------------|------|-------|
| 0x0088C8 - 0x00C791 | 16,074 bytes | Largest EU-unique block |
| 0x00C8D0 - 0x00DAB9 | 4,586 bytes | Data region |
| 0x00DBED - 0x00E8AA | 3,262 bytes | Content block |

### Japan-Unique Regions

| Offset Range | Size | Notes |
|--------------|------|-------|
| 0x009943 - 0x00A59C | 3,162 bytes | Data region |
| 0x00AA12 - 0x00BB3F | 4,398 bytes | Content block |
| 0x00C228 - 0x00E45B | 8,756 bytes | Largest JP-unique block |

## Visual Difference Maps

### Legend
- `.` = Identical
- `+` = <10% different
- `*` = 10-50% different
- `#` = >50% different

### USA vs Europe Pattern
```
0x000000: +######**######**#######...*####.#######+++++###########......+.
0x040000: ######..################*#######################......**......+.
0x080000: .......+.......*....##...*####.......+....+..........+.......+..
0x0C0000: ..............+.......+.......+...............+.......+.......+.
0x100000: ..............*#.......#..............................++......##
```

### USA vs Japan Pattern
```
0x000000: +######**######+*#######...*####.+######.....++.*#######........
0x040000: *#####..#######.#######.#######*#######*########.......*........
0x080000: ................................................................
0x0C0000: ................................................................
```

**Notable:** Banks 4-7 (0x080000-0x0FFFFF) are **100% identical** between USA and Japan!

## Boundary Analysis

### First 32KB (0x0000 - 0x7FFF)
| Comparison | Identical | Different |
|------------|-----------|-----------|
| USA vs EU | 26.63% | 73.37% |
| USA vs JP | 26.63% | 73.37% |
| EU vs JP | **99.81%** | 0.19% |

### Last 32KB (0x1F8000 - 0x1FFFFF)
**100% IDENTICAL** across all three ROMs - contains interrupt vectors and system data.

### Vector Table (Last 64 bytes at 0x1FFFC0)
**100% IDENTICAL** across all three ROMs.

## Recommendations for Editor Development

### 1. Multi-ROM Support Strategy
- **Primary target:** Use USA ROM as reference (most common)
- **Offset compatibility:** All versions share same structure, no pointer adjustments needed
- **Data validation:** Check for presence of expected data signatures

### 2. High-Priority Regions for Regional Support

**For Text/Localization:**
- 0x008000 - 0x00F000: Primary text banks
- 0x048000 - 0x070000: Extended text/data (EU heavily differs here)

**For Game Data (Fighter Stats, etc.):**
- 0x100000 - 0x1F0000: Relatively stable across versions (5-8% difference)
- Use this region for shared game data editing

### 3. Version Detection
```python
def detect_version(rom_data):
    # Check header at 0x7FB0
    region_byte = rom_data[0x7FD9]
    regions = {0x00: 'Japan', 0x01: 'USA', 0x02: 'Europe'}
    return regions.get(region_byte, 'Unknown')
```

### 4. Testing Strategy
1. Test all edits on USA ROM first
2. Verify same offsets work on EU/JP ROMs
3. Validate checksums after edits

## Potential Regional Exclusives

Based on unique difference patterns:

1. **Europe may have additional/censored content** in 0x048000-0x070000 region
2. **Japan may have original text** preserved in some regions where USA/EU diverge
3. **USA and Japan share more content** than either shares with Europe

## Technical Notes

- ROM type: 0x02 = ROM + SRAM
- SRAM size: 8KB (for save data)
- No expansion chips detected (DSP, SuperFX, etc.)
- Standard Nintendo mapper

---

*Report generated on 2026-03-20*
*Tools: Python 3.x, custom ROM comparison scripts*
