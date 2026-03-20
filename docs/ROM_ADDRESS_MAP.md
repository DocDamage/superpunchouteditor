# Super Punch-Out!! ROM Address Map

## USA Version (SHA1: 3604c855790f37db567e9b425252625045f86697)

This document contains all discovered ROM addresses for the USA version of Super Punch-Out!! extracted from the project codebase and known SPO documentation.

---

## Fighter Header Table (Bank $09)

The fighter header table is the master table containing all boxer configuration data. Each entry is 32 bytes.

| Fighter | Index | SNES Address | PC Address | Size |
|---------|-------|--------------|------------|------|
| Gabby Jay | 0 | $09:8000 | 0x048000 | 32 bytes |
| Bear Hugger | 1 | $09:8020 | 0x048020 | 32 bytes |
| Piston Hurricane | 2 | $09:8040 | 0x048040 | 32 bytes |
| Bald Bull | 3 | $09:8060 | 0x048060 | 32 bytes |
| Bob Charlie | 4 | $09:8080 | 0x048080 | 32 bytes |
| Dragon Chan | 5 | $09:80A0 | 0x0480A0 | 32 bytes |
| Masked Muscle | 6 | $09:80C0 | 0x0480C0 | 32 bytes |
| Mr. Sandman | 7 | $09:80E0 | 0x0480E0 | 32 bytes |
| Aran Ryan | 8 | $09:8100 | 0x048100 | 32 bytes |
| Heike Kagero | 9 | $09:8120 | 0x048120 | 32 bytes |
| Mad Clown | 10 | $09:8140 | 0x048140 | 32 bytes |
| Super Macho Man | 11 | $09:8160 | 0x048160 | 32 bytes |
| Narcis Prince | 12 | $09:8180 | 0x048180 | 32 bytes |
| Hoy Quarlow | 13 | $09:81A0 | 0x0481A0 | 32 bytes |
| Rick Bruiser | 14 | $09:81C0 | 0x0481C0 | 32 bytes |
| Nick Bruiser | 15 | $09:81E0 | 0x0481E0 | 32 bytes |

### Shared AI Behavior Headers (Bank $09)

| Shared Group | SNES Address | PC Address | Description |
|--------------|--------------|------------|-------------|
| Gabby Jay / Bob Charlie | $09:8200 | 0x048200 | Shared AI behavior data |
| Bear Hugger / Mad Clown | $09:8220 | 0x048220 | Shared AI behavior data |
| Rick Bruiser / Nick Bruiser | $09:8240 | 0x048240 | Shared AI behavior data |

### Fighter Header Field Offsets (32 bytes total)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 1 byte | palette_id | Palette ID for this fighter |
| 0x01 | 1 byte | attack_power | Attack power rating (0-255) |
| 0x02 | 1 byte | defense_rating | Defense rating (0-255) |
| 0x03 | 1 byte | speed_rating | Speed rating (0-255) |
| 0x06-0x07 | 2 bytes | pose_table_ptr | Pointer to pose/animation table |
| 0x08-0x09 | 2 bytes | ai_script_ptr | Pointer to AI script |
| 0x0A-0x0B | 2 bytes | corner_man_ptr | Pointer to corner man text |

---

## Palette Data (Bank $0D)

Palette data is stored in SNES BGR555 format (2 bytes per color, 16 colors per palette).

| Fighter | SNES Address | PC Address | Size | Description |
|---------|--------------|------------|------|-------------|
| Gabby Jay | $0D:B9DA | 0x06B9DA | 96 bytes | Sprite palette |
| Bear Hugger | $0D:BC3C | 0x06BC3C | 96 bytes | Sprite palette |
| Piston Hurricane | $0D:BE9E | 0x06BE9E | 96 bytes | Sprite palette |
| Bald Bull | $0D:C100 | 0x06C100 | 128 bytes | Sprite palette |
| Bob Charlie | $0D:C382 | 0x06C382 | 96 bytes | Sprite palette |
| Dragon Chan | $0D:C5E4 | 0x06C5E4 | 96 bytes | Sprite palette |
| Masked Muscle | $0D:C846 | 0x06C846 | 128 bytes | Sprite palette |
| Mr. Sandman | $0D:CA88 | 0x06CA88 | 128 bytes | Sprite palette |
| Aran Ryan | $0D:CCCA | 0x06CCCA | 96 bytes | Sprite palette |
| Heike Kagero | $0D:CF2C | 0x06CF2C | 96 bytes | Sprite palette |
| Mad Clown | $0D:D18E | 0x06D18E | 128 bytes | Sprite palette |
| Super Macho Man | $0D:D3F0 | 0x06D3F0 | 96 bytes | Sprite palette |
| Narcis Prince | $0D:D652 | 0x06D652 | 96 bytes | Sprite palette |
| Hoy Quarlow | $0D:D8B4 | 0x06D8B4 | 96 bytes | Sprite palette |
| Rick Bruiser | $0D:DB16 | 0x06DB16 | 128 bytes | Sprite palette |
| Nick Bruiser | $0D:DD58 | 0x06DD58 | 96 bytes | Sprite palette |

---

## Icon/Portrait Graphics

### Small Icons (Bank $0D)

| Fighter | SNES Address | PC Address | Size | Description |
|---------|--------------|------------|------|-------------|
| Gabby Jay | $0D:B7D8 | 0x06B7D8 | 512 bytes | Small icon |
| Bear Hugger | $0D:BA3A | 0x06BA3A | 512 bytes | Small icon |
| Piston Hurricane | $0D:BC9C | 0x06BC9C | 512 bytes | Small icon |
| Bald Bull | $0D:BEFE | 0x06BEFE | 512 bytes | Small icon |
| Bob Charlie | $0D:C180 | 0x06C180 | 512 bytes | Small icon |
| Dragon Chan | $0D:C3E2 | 0x06C3E2 | 512 bytes | Small icon |
| Masked Muscle | $0D:C644 | 0x06C644 | 512 bytes | Small icon |
| Mr. Sandman | $0D:C8A6 | 0x06C8A6 | 512 bytes | Small icon |
| Aran Ryan | $0D:CB08 | 0x06CB08 | 512 bytes | Small icon |
| Heike Kagero | $0D:CD6A | 0x06CD6A | 512 bytes | Small icon |
| Mad Clown | $0D:CFCC | 0x06CFCC | 512 bytes | Small icon |
| Super Macho Man | $0D:D22E | 0x06D22E | 512 bytes | Small icon |
| Narcis Prince | $0D:D490 | 0x06D490 | 512 bytes | Small icon |
| Hoy Quarlow | $0D:D6F2 | 0x06D6F2 | 512 bytes | Small icon |
| Rick Bruiser | $0D:D954 | 0x06D954 | 512 bytes | Small icon |
| Nick Bruiser | $0D:DBB6 | 0x06DBB6 | 512 bytes | Small icon |

### Large Portraits (Bank $10, Compressed)

| Fighter | SNES Address | PC Address | Size | Description |
|---------|--------------|------------|------|-------------|
| Gabby Jay | $10:801C | 0x08001C | 2301 bytes | Large portrait (compressed) |
| Bear Hugger | $10:8919 | 0x080919 | 2362 bytes | Large portrait (compressed) |
| Piston Hurricane | $10:9253 | 0x081253 | 2408 bytes | Large portrait (compressed) |
| Bald Bull | $10:9BBB | 0x081BBB | 2457 bytes | Large portrait (compressed) |
| Bob Charlie | $10:A554 | 0x082554 | 2460 bytes | Large portrait (compressed) |
| Dragon Chan | $10:AEF0 | 0x082EF0 | 2405 bytes | Large portrait (compressed) |
| Masked Muscle | $10:B855 | 0x083855 | 2364 bytes | Large portrait (compressed) |
| Mr. Sandman | $10:C191 | 0x084191 | 2464 bytes | Large portrait (compressed) |
| Aran Ryan | $10:CB31 | 0x084B31 | 2401 bytes | Large portrait (compressed) |
| Heike Kagero | $10:D4B2 | 0x0854B2 | 2446 bytes | Large portrait (compressed) |
| Mad Clown | $10:DE48 | 0x085E48 | 2358 bytes | Large portrait (compressed) |
| Super Macho Man | $10:E7AE | 0x0867AE | 2444 bytes | Large portrait (compressed) |
| Narcis Prince | $10:F13A | 0x08713A | 2378 bytes | Large portrait (compressed) |
| Hoy Quarlow | $10:FAA4 | 0x087AA4 | 2405 bytes | Large portrait (compressed) |
| Rick Bruiser | $11:0419 | 0x088419 | 2428 bytes | Large portrait (compressed) |
| Nick Bruiser | $11:0D85 | 0x088D85 | 2372 bytes | Large portrait (compressed) |

---

## Sprite Data Locations

Sprite data is distributed across multiple banks. Key compressed sprite banks:

### Gabby Jay / Bob Charlie Shared Sprites

| SNES Address | PC Address | Size | Description |
|--------------|------------|------|-------------|
| $3B:8002 | 0x1D8002 | 26006 bytes | Compressed sprite data (GabbyJay2_BobCharlie2) |
| $3C:8002 | 0x1E0002 | 26283 bytes | Compressed sprite data (GabbyJay1_BobCharlie1) |

### Bear Hugger / Mad Clown Shared Sprites

| SNES Address | PC Address | Size | Description |
|--------------|------------|------|-------------|
| $37:8002 | 0x1B8002 | 21969 bytes | Compressed sprite data (BearHugger3_MadClown3) |
| $38:8002 | 0x1C0002 | 21997 bytes | Compressed sprite data (BearHugger2_MadClown2) |
| $39:8002 | 0x1C8002 | 20907 bytes | Compressed sprite data (BearHugger1) |

### Piston Hurricane / Aran Ryan Shared Sprites

| SNES Address | PC Address | Size | Description |
|--------------|------------|------|-------------|
| $33:8002 | 0x198002 | 23722 bytes | Compressed sprite data (PistonHurricane3_AranRyan3) |
| $34:8002 | 0x1A0002 | 24829 bytes | Compressed sprite data (PistonHurricane2_AranRyan2) |
| $35:8002 | 0x1A8002 | 16735 bytes | Compressed sprite data (PistonHurricane1) |

### Bald Bull / Mr. Sandman Shared Sprites

| SNES Address | PC Address | Size | Description |
|--------------|------------|------|-------------|
| $30:8002 | 0x180002 | 24994 bytes | Compressed sprite data (BaldBull3_MrSandman3) |
| $31:8002 | 0x188002 | 26149 bytes | Compressed sprite data (BaldBull2_MrSandman2) |
| $32:8002 | 0x190002 | 25575 bytes | Compressed sprite data (BaldBull1_MrSandman1) |

### Dragon Chan / Heike Kagero Shared Sprites

| SNES Address | PC Address | Size | Description |
|--------------|------------|------|-------------|
| $2E:8002 | 0x170002 | 24466 bytes | Compressed sprite data (DragonChan2_HeikeKagero2) |
| $2F:8002 | 0x178002 | 23796 bytes | Compressed sprite data (DragonChan3) |
| $36:8002 | 0x1B0002 | 6291 bytes | Compressed sprite data (DragonChan1) |

---

## AI Behavior Tables

### AI Script Locations (Bank $09)

Based on KNOWN_SCRIPTS in script-core:

| Fighter | Header SNES | Header PC | AI Data SNES | AI Data PC |
|---------|-------------|-----------|--------------|------------|
| Gabby Jay | $09:8000 | 0x048000 | $09:8800 | 0x048800 |
| Bear Hugger | $09:8020 | 0x048020 | $09:8A00 | 0x048A00 |
| Piston Hurricane | $09:8040 | 0x048040 | $09:8C00 | 0x048C00 |
| Bald Bull | $09:8060 | 0x048060 | $09:8E00 | 0x048E00 |
| Bob Charlie | $09:8080 | 0x048080 | $09:9000 | 0x049000 |
| Dragon Chan | $09:80A0 | 0x0480A0 | $09:9200 | 0x049200 |
| Masked Muscle | $09:80C0 | 0x0480C0 | $09:9400 | 0x049400 |
| Mr. Sandman | $09:80E0 | 0x0480E0 | $09:9600 | 0x049600 |
| Aran Ryan | $09:8100 | 0x048100 | $09:9800 | 0x049800 |
| Heike Kagero | $09:8120 | 0x048120 | $09:9A00 | 0x049A00 |
| Mad Clown | $09:8140 | 0x048140 | $09:9C00 | 0x049C00 |
| Super Macho Man | $09:8160 | 0x048160 | $09:9E00 | 0x049E00 |
| Narcis Prince | $09:8180 | 0x048180 | $09:A000 | 0x04A000 |
| Hoy Quarlow | $09:81A0 | 0x0481A0 | $09:A200 | 0x04A200 |
| Rick Bruiser | $09:81C0 | 0x0481C0 | $09:A400 | 0x04A400 |
| Nick Bruiser | $09:81E0 | 0x0481E0 | $09:A600 | 0x04A600 |

---

## Cornerman Text Data

| Table | SNES Address | PC Address | Description |
|-------|--------------|------------|-------------|
| Corner Man Dialog Table | $09:9000 | 0x049000 | Table of corner man message pointers |

### Player Data

| Data | SNES Address | PC Address | Description |
|------|--------------|------------|-------------|
| Little Mac Script | $09:A000 | 0x04A000 | Player movement and attack script |

---

## Audio Data (SPC700)

### SPC File Format Offsets (within SPC file)

| Offset | Size | Description |
|--------|------|-------------|
| 0x00 | 256 bytes | SPC header |
| 0x100 | 65536 bytes | SPC700 RAM |
| 0x10100 | 128 bytes | DSP registers |
| 0x10180 | 64 bytes | Extra RAM |
| 0x10200 | - | Total SPC file size |

### Audio Assets in ROM (from boxers_usa.json)

The manifest lists 35 music tracks and 122 samples in the SPC700 category. Exact ROM locations are scattered throughout banks $0B-$0F and require further research.

---

## SNES Header Information

| Field | PC Offset | Size | Description |
|-------|-----------|------|-------------|
| Title | 0x007FC0 | 21 bytes | Game title "SUPER PUNCH-OUT!!" |
| Map/Type | 0x007FD5 | 1 byte | ROM mapping mode |
| ROM Size | 0x007FD7 | 1 byte | ROM size indicator |
| Version | 0x007FDB | 1 byte | Version byte |

Alternative header location (with SMC header): 0x0081C0

---

## Circuit and Roster Data

### Circuit Assignments

| Circuit | Fighter IDs | Description |
|---------|-------------|-------------|
| Minor Circuit | 0, 1, 2, 3 | Gabby Jay, Bear Hugger, Piston Hurricane, Bald Bull |
| Major Circuit | 4, 5, 6, 7 | Bob Charlie, Dragon Chan, Masked Muscle, Mr. Sandman |
| World Circuit | 8, 9, 10, 11 | Aran Ryan, Heike Kagero, Mad Clown, Super Macho Man |
| Special Circuit | 12, 13, 14, 15 | Narcis Prince, Hoy Quarlow, Rick Bruiser, Nick Bruiser |

### Table Locations (RESEARCH NEEDED)

| Table | Expected Bank | Status |
|-------|---------------|--------|
| Boxer Names Table | $0C | ❌ Address unknown |
| Circuit Assignment Table | $0C | ❌ Address unknown |
| Unlock Order Table | $0C | ❌ Address unknown |
| Intro Text Table | $0C | ❌ Address unknown |
| Victory Quotes Table | $0C | ❌ Address unknown |

---

## Known Free Space Regions

| PC Start | PC End | Size | Description |
|----------|--------|------|-------------|
| 0x007FC0 | 0x008000 | 64 bytes | Extended header area (sometimes unused) |

---

## Research Gaps

The following addresses are still unknown and require ROM analysis:

### Critical Unknown Addresses

| Category | Priority | Notes |
|----------|----------|-------|
| Boxer Names Table | HIGH | Fighter name strings for display |
| Circuit Assignment Table | HIGH | Determines circuit membership |
| Unlock Order Table | MEDIUM | Controls unlock progression |
| Intro Text Pointers | MEDIUM | Pre-fight boxer introductions |
| Victory Quotes | MEDIUM | Post-fight victory/defeat text |
| Menu Text | LOW | UI text strings |
| Credits Text | LOW | End credits |

### Audio Data Gaps

| Category | Priority | Notes |
|----------|----------|-------|
| SPC Engine Location | MEDIUM | Main audio driver |
| Sample Table | MEDIUM | BRR sample pointers |
| Music Sequence Table | MEDIUM | Song data locations |

### AI Data Gaps

| Category | Priority | Notes |
|----------|----------|-------|
| Pattern Data Locations | HIGH | Per-boxer attack patterns |
| Defense Behavior Tables | HIGH | Blocking/dodging logic |
| Animation Script Tables | MEDIUM | Pose sequencing |

---

## Address Conversion Reference

### LoROM Mapping Formula

```
PC to SNES:
  Bank = (PC / 0x8000) | 0x80
  Address = (PC % 0x8000) | 0x8000

SNES to PC:
  PC = (Bank & 0x7F) * 0x8000 + (Address & 0x7FFF)
```

### Common Bank Ranges

| Bank Range | PC Range | Typical Content |
|------------|----------|-----------------|
| $80-$87 | 0x000000-0x03FFFF | Code, low data |
| $88-$8F | 0x040000-0x07FFFF | Fighter headers, AI |
| $90-$9F | 0x080000-0x0BFFFF | Graphics, portraits |
| $A0-$AF | 0x0C0000-0x13FFFF | Sprite data |
| $B0-$BF | 0x140000-0x1BFFFF | Compressed sprites |
| $C0-$CF | 0x1C0000-0x23FFFF | More compressed data |
| $D0-$DF | 0x240000-0x2BFFFF | Audio, misc data |

---

## Version Differences

### USA Version (Supported)
- SHA1: `3604c855790f37db567e9b425252625045f86697`
- Size: 2,097,152 bytes (2MB)
- All addresses in this document apply to this version

### JPN Version (Research Needed)
- SHA1: Unknown
- Expected differences: Text encoding, possibly different text table location

### PAL Version (Research Needed)
- SHA1: Unknown
- Expected differences: Timing adjustments, multi-language text

---

## Source Files

This address map was compiled from:
- `data/manifests/boxers_usa.json` - Asset locations
- `crates/script-core/src/lib.rs` - KNOWN_SCRIPTS table
- `crates/script-core/src/ai_behavior.rs` - AI script locations
- `crates/rom-core/src/region.rs` - Region configuration
- `crates/rom-core/src/roster.rs` - Roster data structures
- `crates/rom-core/src/text.rs` - Text system documentation

---

*Last updated: 2026-03-19*
*Document version: 1.0*
