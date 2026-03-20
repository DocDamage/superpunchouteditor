"""Entry point for the Super Punch‑Out!! editor CLI.

This script simply dispatches to the ``cli`` object defined in
``superpunchouteditor.editor`` so that ``python supo.py`` works without
installing the package.  When installing via ``pip`` the ``supo`` console
script is registered automatically.
"""

from superpunchouteditor.editor import cli


if __name__ == "__main__":  # pragma: no cover
    cli()