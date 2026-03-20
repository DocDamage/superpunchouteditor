"""Top‑level package for Super Punch‑Out!! editor.

This package provides a framework for loading a Super Punch‑Out!! ROM,
parsing out fighter data, and writing modifications back.  See
``README.md`` for high‑level documentation.
"""

from importlib.metadata import version

__all__ = ["Rom", "Fighter", "fighters"]

try:
    __version__ = version("superpunchouteditor")
except Exception:
    __version__ = "0.0.0"

from .rom import Rom  # noqa: E402  (import after __version__)
from .fighter import Fighter, fighters  # noqa: E402