"""High‑level fighter model and utility functions.

This module defines a :class:`Fighter` class that knows how to extract and
modify fighter data (names and stats) from a loaded ROM.  It leverages
offsets defined in :mod:`superpunchouteditor.constants` to read and write
specific byte ranges.  A simple registry of all fighters is provided for
easy iteration.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import List

from .rom import Rom
from .constants import FighterEntry, FIGHTERS


@dataclass
class Fighter:
    """Represents a fighter entry within the ROM.

    Provides convenience methods for reading and writing the fighter's name
    and stats.  All offsets and lengths are defined by the underlying
    :class:`FighterEntry` descriptor.
    """

    entry: FighterEntry

    def get_name(self, rom: Rom) -> str:
        """Return the fighter's name from the ROM, decoded as ASCII.

        Any trailing null bytes or spaces are stripped.  If decoding
        encounters non‑ASCII bytes, they are ignored.
        """
        raw = rom.get_bytes(self.entry.name_offset, self.entry.name_length)
        # Strip trailing nulls and spaces, then decode ignoring errors
        name_bytes = raw.rstrip(b"\x00 ")
        return name_bytes.decode("ascii", errors="ignore")

    def set_name(self, rom: Rom, name: str) -> None:
        """Write a new name for the fighter into the ROM.

        The name is encoded to ASCII, truncated to ``name_length`` bytes if
        necessary, and padded with spaces up to the declared length.  If the
        supplied ``name`` contains non‑ASCII characters they will be ignored.
        """
        encoded = name.encode("ascii", errors="ignore")[: self.entry.name_length]
        padded = encoded.ljust(self.entry.name_length, b" ")
        rom.set_bytes(self.entry.name_offset, padded)

    def get_stats(self, rom: Rom) -> bytes:
        """Return the raw stat block for the fighter.

        The meaning of these bytes is game‑specific and left to the user to
        interpret.
        """
        return rom.get_bytes(self.entry.stats_offset, self.entry.stats_length)

    def set_stats(self, rom: Rom, stats: bytes) -> None:
        """Overwrite the stat block for the fighter.

        :param stats: A byte sequence of exactly ``stats_length`` bytes.
        :raises ValueError: If ``stats`` is not the correct length.
        """
        if len(stats) != self.entry.stats_length:
            raise ValueError(
                f"Stat block must be {self.entry.stats_length} bytes (got {len(stats)})"
            )
        rom.set_bytes(self.entry.stats_offset, stats)


def load_fighters() -> List[Fighter]:
    """Return a list of :class:`Fighter` objects for all defined entries."""
    return [Fighter(entry) for entry in FIGHTERS]


# Pre‑instantiate fighters for convenience
fighters: List[Fighter] = load_fighters()

__all__ = ["Fighter", "fighters", "load_fighters"]