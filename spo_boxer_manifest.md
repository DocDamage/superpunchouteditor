# Super Punch-Out!! asset extraction + boxer manifest

- ROM: `Super Punch-Out!! (USA).sfc`
- SHA-1: `3604c855790f37db567e9b425252625045f86697`
- Size: `2,097,152` bytes

## What was extracted
- Graphics: **393** files
- Graphics/Compressed: **83** files
- Tilemaps/Compressed: **1** files
- Palettes: **39** files
- GarbageData: **13** files
- SPC700/Music: **35** files
- SPC700/Samples: **122** files

## Shared sprite-bank pairs
- **Aran Ryan** ↔ **Piston Hurricane**
- **Bald Bull** ↔ **Mr. Sandman**
- **Bear Hugger** ↔ **Mad Clown**
- **Bob Charlie** ↔ **Gabby Jay**
- **Dragon Chan** ↔ **Heike Kagero**
- **Masked Muscle** ↔ **Super Macho Man**
- **Nick Bruiser** ↔ **Rick Bruiser**

## Safest full-body test targets
- **Hoy Quarlow** — 47 unique sprite bins, 0 shared sprite bins
- **Narcis Prince** — 31 unique sprite bins, 0 shared sprite bins

## Lowest-risk mod types for any boxer
- Palette swap
- Icon swap
- Large portrait swap

## Boxer summary

| Fighter | Ref sheet | Palette | Icon | Large portrait | Unique sprite bins | Shared sprite bins | Shared with |
|---|---|---:|---:|---:|---:|---:|---|
| Gabby Jay | `sprites/Gabby Jay.png` | 1 | 1 | 1 | 0 | 18 | Bob Charlie |
| Bear Hugger | `sprites/Bear Hugger.png` | 1 | 1 | 1 | 1 | 49 | Mad Clown |
| Piston Hurricane | `sprites/Piston Hurricane.png` | 1 | 1 | 1 | 1 | 25 | Aran Ryan |
| Bald Bull | `sprites/Bald Bull.png` | 1 | 1 | 1 | 0 | 45 | Mr. Sandman |
| Bob Charlie | `sprites/Bob Charlie.png` | 1 | 1 | 1 | 1 | 18 | Gabby Jay |
| Dragon Chan | `sprites/Dragon Chan.png` | 1 | 1 | 1 | 2 | 27 | Heike Kagero |
| Masked Muscle | `sprites/Masked Muscle.png` | 1 | 1 | 1 | 2 | 52 | Super Macho Man |
| Mr. Sandman | `sprites/Mr. Sandman.png` | 1 | 1 | 1 | 0 | 45 | Bald Bull |
| Aran Ryan | `sprites/Aran Ryan.png` | 1 | 1 | 1 | 1 | 25 | Piston Hurricane |
| Heike Kagero | `sprites/Heike Kagero.png` | 1 | 1 | 1 | 2 | 27 | Dragon Chan |
| Mad Clown | `sprites/Mad Clown.png` | 1 | 1 | 1 | 2 | 49 | Bear Hugger |
| Super Macho Man | `sprites/Super Macho Man.png` | 1 | 1 | 1 | 1 | 52 | Masked Muscle |
| Narcis Prince | `sprites/Narcis Prince.png` | 1 | 1 | 1 | 31 | 0 | — |
| Hoy Quarlow | `sprites/Hoy Quarlow.png` | 1 | 1 | 1 | 47 | 0 | — |
| Rick Bruiser | `sprites/Rick Bruiser.png` | 1 | 1 | 1 | 1 | 51 | Nick Bruiser |
| Nick Bruiser | `sprites/Nick Bruiser.png` | 1 | 1 | 1 | 1 | 51 | Rick Bruiser |
| Little Mac | `sprites/Little Mac.png` | 0 | 0 | 0 | 1 | 0 | — |

## Per-boxer key files
### Gabby Jay
- Reference sheet: `sprites/Gabby Jay.png`
- Palette: `Palettes/Sprite_GabbyJay.bin`
- Icon: `Graphics/GFX_Sprite_GabbyJayIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_GabbyJayLargePortrait.bin`
- Unique sprite bins: **0**
- Shared sprite bins: **18**
- Shared with: **Bob Charlie**
- Example shared bins:
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex4E.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex6D.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex48.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex59.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex64.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex65.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex04.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex05.bin`
  - _... plus 10 more in the JSON manifest_

### Bear Hugger
- Reference sheet: `sprites/Bear Hugger.png`
- Palette: `Palettes/Sprite_BearHugger.bin`
- Icon: `Graphics/GFX_Sprite_BearHuggerIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_BearHuggerLargePortrait.bin`
- Unique sprite bins: **1**
- Shared sprite bins: **49**
- Shared with: **Mad Clown**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_BearHugger1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2B.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2C.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2D.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2E.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2F.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex30.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex31.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex34.bin`
  - _... plus 41 more in the JSON manifest_

### Piston Hurricane
- Reference sheet: `sprites/Piston Hurricane.png`
- Palette: `Palettes/Sprite_PistonHurricane.bin`
- Icon: `Graphics/GFX_Sprite_PistonHurricaneIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_PistonHurricaneLargePortrait.bin`
- Unique sprite bins: **1**
- Shared sprite bins: **25**
- Shared with: **Aran Ryan**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_PistonHurricane1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex7F.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex49.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex4A.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex27.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex28.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex48.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex50.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex03.bin`
  - _... plus 17 more in the JSON manifest_

### Bald Bull
- Reference sheet: `sprites/Bald Bull.png`
- Palette: `Palettes/Sprite_BaldBull.bin`
- Icon: `Graphics/GFX_Sprite_BaldBullIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_BaldBullLargePortrait.bin`
- Unique sprite bins: **0**
- Shared sprite bins: **45**
- Shared with: **Mr. Sandman**
- Example shared bins:
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex70.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex4E.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex4F.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex50.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex51.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex52.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex53.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex54.bin`
  - _... plus 37 more in the JSON manifest_

### Bob Charlie
- Reference sheet: `sprites/Bob Charlie.png`
- Palette: `Palettes/Sprite_BobCharlie.bin`
- Icon: `Graphics/GFX_Sprite_BobCharlieIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_BobCharlieLargePortrait.bin`
- Unique sprite bins: **1**
- Shared sprite bins: **18**
- Shared with: **Gabby Jay**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_BobCharlie3.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex4E.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex6D.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex48.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex59.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex64.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex65.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex04.bin`
  - `Graphics/GFX_Sprite_GabbyJayBobCharlieIndex05.bin`
  - _... plus 10 more in the JSON manifest_

### Dragon Chan
- Reference sheet: `sprites/Dragon Chan.png`
- Palette: `Palettes/Sprite_DragonChan.bin`
- Icon: `Graphics/GFX_Sprite_DragonChanIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_DragonChanLargePortrait.bin`
- Unique sprite bins: **2**
- Shared sprite bins: **27**
- Shared with: **Heike Kagero**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_DragonChan3.bin`
  - `Graphics/Compressed/GFX_Sprite_DragonChan1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex9E.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex9F.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex45.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex46.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex79.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex80.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex81.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex87.bin`
  - _... plus 19 more in the JSON manifest_

### Masked Muscle
- Reference sheet: `sprites/Masked Muscle.png`
- Palette: `Palettes/Sprite_MaskedMuscle.bin`
- Icon: `Graphics/GFX_Sprite_MaskedMuscleIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_MaskedMuscleLargePortrait.bin`
- Unique sprite bins: **2**
- Shared sprite bins: **52**
- Shared with: **Super Macho Man**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_MaskedMuscleSpit.bin`
  - `Graphics/Compressed/GFX_Sprite_MaskedMuscle1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex90.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex03.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex04.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex4D.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex4E.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex51.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex52.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex53.bin`
  - _... plus 44 more in the JSON manifest_

### Mr. Sandman
- Reference sheet: `sprites/Mr. Sandman.png`
- Palette: `Palettes/Sprite_MrSandman.bin`
- Icon: `Graphics/GFX_Sprite_MrSandmanIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_MrSandmanLargePortrait.bin`
- Unique sprite bins: **0**
- Shared sprite bins: **45**
- Shared with: **Bald Bull**
- Example shared bins:
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex70.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex4E.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex4F.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex50.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex51.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex52.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex53.bin`
  - `Graphics/GFX_Sprite_BaldBullMrSandmanIndex54.bin`
  - _... plus 37 more in the JSON manifest_

### Aran Ryan
- Reference sheet: `sprites/Aran Ryan.png`
- Palette: `Palettes/Sprite_AranRyan.bin`
- Icon: `Graphics/GFX_Sprite_AranRyanIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_AranRyanLargePortrait.bin`
- Unique sprite bins: **1**
- Shared sprite bins: **25**
- Shared with: **Piston Hurricane**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_AranRyan1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex7F.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex49.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex4A.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex27.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex28.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex48.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex50.bin`
  - `Graphics/GFX_Sprite_PistonHurricaneAranRyanIndex03.bin`
  - _... plus 17 more in the JSON manifest_

### Heike Kagero
- Reference sheet: `sprites/Heike Kagero.png`
- Palette: `Palettes/Sprite_HeikeKagero.bin`
- Icon: `Graphics/GFX_Sprite_HeikeKageroIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_HeikeKageroLargePortrait.bin`
- Unique sprite bins: **2**
- Shared sprite bins: **27**
- Shared with: **Dragon Chan**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_HeikeKagero3.bin`
  - `Graphics/Compressed/GFX_Sprite_HeikeKagero1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex9E.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex9F.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex45.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex46.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex79.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex80.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex81.bin`
  - `Graphics/GFX_Sprite_DragonChanHeikeKageroIndex87.bin`
  - _... plus 19 more in the JSON manifest_

### Mad Clown
- Reference sheet: `sprites/Mad Clown.png`
- Palette: `Palettes/Sprite_MadClown.bin`
- Icon: `Graphics/GFX_Sprite_MadClownIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_MadClownLargePortrait.bin`
- Unique sprite bins: **2**
- Shared sprite bins: **49**
- Shared with: **Bear Hugger**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_MadClownBall.bin`
  - `Graphics/Compressed/GFX_Sprite_MadClown1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2B.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2C.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2D.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2E.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex2F.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex30.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex31.bin`
  - `Graphics/GFX_Sprite_BearHuggerMadClownIndex34.bin`
  - _... plus 41 more in the JSON manifest_

### Super Macho Man
- Reference sheet: `sprites/Super Macho Man.png`
- Palette: `Palettes/Sprite_SuperMachoMan.bin`
- Icon: `Graphics/GFX_Sprite_SuperMachoManIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_SuperMachoManLargePortrait.bin`
- Unique sprite bins: **1**
- Shared sprite bins: **52**
- Shared with: **Masked Muscle**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_SuperMachoMan1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex90.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex03.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex04.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex4D.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex4E.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex51.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex52.bin`
  - `Graphics/GFX_Sprite_MaskedMuscleSuperMachoManIndex53.bin`
  - _... plus 44 more in the JSON manifest_

### Narcis Prince
- Reference sheet: `sprites/Narcis Prince.png`
- Palette: `Palettes/Sprite_NarcisPrince.bin`
- Icon: `Graphics/GFX_Sprite_NarcisPrinceIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_NarcisPrinceLargePortrait.bin`
- Unique sprite bins: **31**
- Shared sprite bins: **0**
- Example unique bins:
  - `Graphics/GFX_Sprite_NarcisPrinceIndex0D.bin`
  - `Graphics/GFX_Sprite_NarcisPrinceIndex60.bin`
  - `Graphics/GFX_Sprite_NarcisPrinceIndex03.bin`
  - `Graphics/GFX_Sprite_NarcisPrinceIndex04.bin`
  - `Graphics/GFX_Sprite_NarcisPrinceIndex2A.bin`
  - `Graphics/GFX_Sprite_NarcisPrinceIndex2B.bin`
  - `Graphics/GFX_Sprite_NarcisPrinceIndex2C.bin`
  - `Graphics/GFX_Sprite_NarcisPrinceIndex39.bin`
  - _... plus 23 more in the JSON manifest_

### Hoy Quarlow
- Reference sheet: `sprites/Hoy Quarlow.png`
- Palette: `Palettes/Sprite_HoyQuarlow.bin`
- Icon: `Graphics/GFX_Sprite_HoyQuarlowIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_HoyQuarlowLargePortrait.bin`
- Unique sprite bins: **47**
- Shared sprite bins: **0**
- Example unique bins:
  - `Graphics/GFX_Sprite_HoyQuarlowIndex25.bin`
  - `Graphics/GFX_Sprite_HoyQuarlowIndex26.bin`
  - `Graphics/GFX_Sprite_HoyQuarlowIndex27.bin`
  - `Graphics/GFX_Sprite_HoyQuarlowIndex46.bin`
  - `Graphics/GFX_Sprite_HoyQuarlowIndex47.bin`
  - `Graphics/GFX_Sprite_HoyQuarlowIndex48.bin`
  - `Graphics/GFX_Sprite_HoyQuarlowIndex5F.bin`
  - `Graphics/GFX_Sprite_HoyQuarlowIndex60.bin`
  - _... plus 39 more in the JSON manifest_

### Rick Bruiser
- Reference sheet: `sprites/Rick Bruiser.png`
- Palette: `Palettes/Sprite_RickBruiser.bin`
- Icon: `Graphics/GFX_Sprite_RickBruiserIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_RickBruiserLargePortrait.bin`
- Unique sprite bins: **1**
- Shared sprite bins: **51**
- Shared with: **Nick Bruiser**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_RickBruiser1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex6A.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex6B.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex41.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex42.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex43.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex12.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex13.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex14.bin`
  - _... plus 43 more in the JSON manifest_

### Nick Bruiser
- Reference sheet: `sprites/Nick Bruiser.png`
- Palette: `Palettes/Sprite_NickBruiser.bin`
- Icon: `Graphics/GFX_Sprite_NickBruiserIcon.bin`
- Large portrait: `Graphics/Compressed/GFX_Sprite_NickBruiserLargePortrait.bin`
- Unique sprite bins: **1**
- Shared sprite bins: **51**
- Shared with: **Rick Bruiser**
- Example unique bins:
  - `Graphics/Compressed/GFX_Sprite_NickBruiser1.bin`
- Example shared bins:
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex6A.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex6B.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex41.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex42.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex43.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex12.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex13.bin`
  - `Graphics/GFX_Sprite_RickBruiserNickBruiserIndex14.bin`
  - _... plus 43 more in the JSON manifest_

### Little Mac
- Reference sheet: `sprites/Little Mac.png`
- Unique sprite bins: **1**
- Shared sprite bins: **0**
- Example unique bins:
  - `Graphics/GFX_Player_BoxingGloves.bin`

## Notes
- Sprite/icon/portrait/palette ownership was inferred from extracted asset filenames defined by AssetPointersAndFiles.asm.
- A file was marked as shared when its filename included more than one boxer name (for example PistonHurricaneAranRyan).
- Large portraits are stored in compressed graphics bins but are broken out separately in the manifest.
- Narcis Prince and Hoy Quarlow are the cleanest full-body test targets because they have zero shared sprite bins in the filename-based manifest.