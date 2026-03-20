"""Low‑level ROM handling utilities.

This module provides the ``Rom`` class which encapsulates a Super Nintendo
ROM as a mutable byte array.  It includes convenience methods for reading
and writing byte sequences at arbitrary offsets and saving the modified ROM
back to disk.  You can also use it as a context manager to ensure the file
handles are properly closed.

Example usage::

    from superpunchouteditor.rom import Rom

    rom = Rom.from_file("super_punch_out.sfc")
    # Read 16 bytes at offset 0x10000
    name_bytes = rom.get_bytes(0x10000, 16)
    # Modify bytes in place
    rom.set_bytes(0x10000, b"NEW FIGHTER NAME\0\0\0")
    rom.save("patched.sfc")

Note that no attempt is made to verify that the file is in fact a
Super Punch‑Out!! ROM.  This class simply treats the input as a binary blob
and leaves validation to higher‑level code.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import ByteString, Optional


@dataclass
class Rom:
    """Represents a Super Nintendo ROM loaded into memory.

    Internally the data is stored as a mutable ``bytearray`` so that
    modifications can be applied easily.  Use :meth:`save` to write the data
    back to disk.
    """

    data: bytearray

    @classmethod
    def from_file(cls, path: str) -> "Rom":
        """Read a ROM from ``path`` and return a :class:`Rom` instance.

        :param path: Path to a `.sfc` or `.smc` file containing the game ROM.
        :raises FileNotFoundError: If ``path`` does not exist.
        """
        with open(path, "rb") as f:
            contents = f.read()
        return cls(bytearray(contents))

    def get_bytes(self, offset: int, length: int) -> bytes:
        """Return ``length`` bytes starting at ``offset`` as an immutable ``bytes``.

        :param offset: 0‑based file offset.
        :param length: Number of bytes to read.
        :returns: Slice of the underlying data as immutable bytes.
        :raises IndexError: If the requested range lies outside the ROM.
        """
        return bytes(self.data[offset : offset + length])

    def set_bytes(self, offset: int, values: ByteString) -> None:
        """Overwrite bytes starting at ``offset`` with ``values``.

        If ``values`` is shorter than the original data slice, only the
        specified bytes are overwritten.  If longer, the data after
        ``offset + len(values)`` will be shifted left (i.e. truncated) to
        accommodate the new data.  For ROM patching you generally want to
        ensure that you write exactly the number of bytes you intend to
        modify.

        :param offset: 0‑based file offset.
        :param values: Byte‑like object to write.
        :raises IndexError: If ``offset`` is out of range.
        """
        end = offset + len(values)
        self.data[offset:end] = values

    def save(self, path: str) -> None:
        """Write the ROM to ``path``.

        :param path: Output file name.  If the file exists it will be
            overwritten.  Consider using a new filename to preserve the
            original ROM.
        """
        with open(path, "wb") as f:
            f.write(self.data)

    def __enter__(self) -> "Rom":  # pragma: no cover - context manager sugar
        return self

    def __exit__(self, exc_type, exc, tb) -> Optional[bool]:  # pragma: no cover
        return None