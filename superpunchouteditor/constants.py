"""Constants used by the Super Punch‑Out!! editor.

This file defines offsets and lengths for various data structures within the
Super Punch‑Out!! ROM.  The values provided here are **placeholders**: they
illustrate how to describe a table of fighter names but do not reflect the
actual layout of the ROM.  To make the editor useful, you will need to
determine the correct offsets by consulting documentation or reverse
engineering the game.  Once known, update these values accordingly.
"""

from dataclasses import dataclass
from typing import List


@dataclass
class FighterEntry:
    """Describes where a fighter's name and stats are stored within the ROM.

    :param label: Human‑readable label used in the CLI (e.g. "Gabby Jay").
    :param name_offset: File offset in bytes where the fighter's name string
        begins.  Offsets are relative to the start of the ROM file (not
        SNES memory addresses).
    :param name_length: Number of bytes allocated for the fighter's name.  If
        the new name is shorter than this length, remaining bytes will be
        padded with spaces; if longer, it will be truncated.
    :param stats_offset: File offset where the fighter's stat block begins.
    :param stats_length: Length of the fighter's stat block in bytes.
    """

    label: str
    name_offset: int
    name_length: int
    stats_offset: int
    stats_length: int


# Default placeholder entries.  Each entry reserves 16 bytes for the name
# string and 8 bytes for the stats (both arbitrary).  Offsets are
# sequentially increasing purely for demonstration purposes.  Replace these
# offsets and lengths once you determine the real data layout.
_BASE_NAME_OFFSET = 0x10000  # arbitrary starting offset for names
_BASE_STATS_OFFSET = 0x20000  # arbitrary starting offset for stats
_NAME_STEP = 0x20  # allocate 32 bytes between names (16 for name + padding)
_STATS_STEP = 0x10  # allocate 16 bytes between stat blocks (8 for stats + padding)


# List of fighters in order of appearance.  You can add or remove entries as
# needed.  The offsets below are calculated relative to the base offsets and
# are placeholders.
fighter_labels = [
    "Gabby Jay",
    "Bear Hugger",
    "Piston Hurricane",
    "Bald Bull",
    "Bob Charlie",
    "Dragon Chan",
    "Masked Muscle",
    "Mr. Sandman",
    "Aran Ryan",
    "Heike Kagero",
    "Mad Clown",
    "Super Macho Man",
    "Narcis Prince",
    "Hoy Quarlow",
    "Rick Bruiser",
    "Nick Bruiser",
]


FIGHTERS: List[FighterEntry] = []

for i, label in enumerate(fighter_labels):
    name_offset = _BASE_NAME_OFFSET + i * _NAME_STEP
    stats_offset = _BASE_STATS_OFFSET + i * _STATS_STEP
    FIGHTERS.append(
        FighterEntry(
            label=label,
            name_offset=name_offset,
            name_length=16,
            stats_offset=stats_offset,
            stats_length=8,
        )
    )


__all__ = ["FighterEntry", "FIGHTERS", "fighter_labels"]