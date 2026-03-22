# Super Punch-Out!! ROM Text Encoding Analysis

## ROM Files Analyzed
- `Super Punch-Out!! (USA).sfc` - 2,097,152 bytes (2MB)
- `Super Punch-Out!! (Europe).sfc` - 2,097,152 bytes (2MB)  
- `Super Punch-Out!! (Japan) (NP).sfc` - 2,097,152 bytes (2MB)

## ROM Header Information (SNES Internal Header at 0x7FC0)
| Field | USA | Europe | Japan |
|-------|-----|--------|-------|
| Game Title | "Super Punch-Out!!" | "Super Punch-Out!!" | "Super Punch-Out!!" |
| Map Mode | 0x20 (LoROM) | 0x20 (LoROM) | 0x20 (LoROM) |
| ROM Type | 0x02 (ROM+RAM) | 0x02 (ROM+RAM) | 0x02 (ROM+RAM) |
| ROM Size | 0x0B (4MB) | 0x0B (4MB) | 0x0B (4MB) |
| Destination | 0x01 (North America) | 0x02 (Europe) | 0x00 (Japan) |

## ROM Comparison Summary
| Comparison | Different Bytes | Percentage |
|------------|-----------------|------------|
| USA vs JPN | 378,245 | 18.04% |
| USA vs EU | 441,861 | 21.07% |
| JPN vs EU | 397,326 | 18.95% |

## Text Encoding Findings

### 1. **Text is NOT stored as plain ASCII**
- Boxer names like "GLASS JOE", "BALD BULL", etc. are NOT found as plain ASCII strings
- Text appears to be **compressed or custom-encoded**

### 2. **Partial Text Pattern Found**
- Found encoding pattern: **A=0x00, B=0x01, C=0x02, ..., Z=0x19** in some regions
- Example: "JOE" found at offset 0xB6E4A as bytes `09 0E 04`
- "BALD" found at offset 0xFB178 as bytes `01 00 0B 03`
- "MAD" found at offset 0x417B7

### 3. **Key Text Table Regions** (USA-JPN Differences)
| Address Range | Size | Description |
|---------------|------|-------------|
| 0x3614E - 0x36628 | 1,243 bytes | Major text region |
| 0x13F288 - 0x13F67E | 1,015 bytes | Major text region |
| 0x27BE3 - 0x27F84 | 930 bytes | Text region |
| 0x32641 - 0x329BF | 895 bytes | Text region |
| 0x37C12 - 0x37F6D | 860 bytes | Text region |
| 0x14F627 - 0x14F910 | 746 bytes | Text region |

### 4. **Key Text Table Regions** (USA-EU Differences)
| Address Range | Size | Description |
|---------------|------|-------------|
| 0x4E955 - 0x4FFAA | 5,718 bytes | Largest EU text region |
| 0x56E81 - 0x58000 | 4,480 bytes | Large EU text region |
| 0x67043 - 0x68000 | 4,030 bytes | EU text region |

### 5. **Byte Distribution Analysis**

**USA-JPN Text Region (0x3614E):**
- High bytes (0x80-0xFF): 48.1%
- Low bytes (0x00-0x7F): 51.9%
- Top bytes: 0x00, 0xFF, 0xFE, 0x10, 0x01
- Suggests: Compressed data with control codes

**USA-EU Text Region (0x4E955):**
- High bytes (0x80-0xFF): 73.6%
- Low bytes (0x00-0x7F): 26.4%
- Top bytes: 0xAA, 0xEA, 0xA2, 0xA8, 0x2A
- Suggests: Pattern-heavy compressed data

### 6. **Japanese Text Encoding**
- Shift-JIS patterns detected but **NOT valid Shift-JIS text**
- Japanese ROM has many high-byte sequences that don't decode properly
- Likely uses a **custom encoding or compression** for Japanese characters

## Encoding Characteristics

### Detected Control Codes:
- `0x00` - Common terminator or separator
- `0xFF` - Common terminator
- `0xFE` - Possible punctuation (period) or control code
- `0x10`, `0x01` - Frequent control codes

### Text Storage Pattern:
1. **Compressed/Encoded Data**: Text is stored in compressed blocks
2. **Pointer Tables**: Text is accessed via pointer tables (found at 0x60D0)
3. **Regional Variations**: Major differences between USA/EU/JPN versions

## Boxer Names Status

| Boxer Name | Status | Notes |
|------------|--------|-------|
| GLASS JOE | NOT FOUND (plain) | Likely encoded |
| PISTON HURRICANE | NOT FOUND (plain) | Likely encoded |
| BALD BULL | NOT FOUND (plain) | Likely encoded |
| KING HIPPO | NOT FOUND (plain) | Likely encoded |
| BEAR HUGGER | NOT FOUND (plain) | Likely encoded |
| DRAGON CHAN | NOT FOUND (plain) | Likely encoded |
| MASKED MUSCLE | NOT FOUND (plain) | Likely encoded |
| MR. SANDMAN | NOT FOUND (plain) | Likely encoded |
| ARDY | NOT FOUND (plain) | Likely encoded |
| HEIKE KAGERO | NOT FOUND (plain) | Likely encoded |
| MAD CLOWN | NOT FOUND (plain) | Likely encoded |
| SUPER MACHO MAN | NOT FOUND (plain) | Likely encoded |
| NARCIS PRINCE | NOT FOUND (plain) | Likely encoded |
| HOY QUARLOW | NOT FOUND (plain) | Likely encoded |
| RICK BRUISER | NOT FOUND (plain) | Likely encoded |
| NICK BRUISER | NOT FOUND (plain) | Likely encoded |

## Recommendations for Text Editing

1. **Locate Decompression Routine**: The game must decompress text at runtime. Finding this routine would help understand the encoding.

2. **Use Relative Search**: Since boxer names appear to use A=0, B=1 encoding, a relative search tool could find name tables.

3. **Compare Same-Screen Text**: Comparing text that appears on the same screen across ROM versions would help identify corresponding text blocks.

4. **Tile-Based Approach**: Text might be stored as tile indices rather than character codes. Analyzing the game's font/tile data would help.

5. **Manifest Structure**: For manifest creation, treat each ROM version separately with their identified text regions:
   - USA: Regions at 0x3614E, 0x13F288, etc.
   - EU: Regions at 0x4E955, 0x56E81, etc.
   - JPN: Regions at 0x3614E (different content), etc.

## Summary

The Super Punch-Out!! ROMs use a **custom text encoding/compression system** that is not plain ASCII or standard Shift-JIS. Text is stored in compressed blocks with significant regional variations between USA, European, and Japanese versions. The encoding appears to use:
- A=0x00, B=0x01, C=0x02... letter encoding in some contexts
- Control codes (0x00, 0xFF, 0xFE) for terminators/punctuation
- Compression algorithms that produce high-byte-heavy output

Further reverse engineering of the decompression routine would be needed to fully decode the text.
