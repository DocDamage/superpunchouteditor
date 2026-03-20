"""Command line interface for Super Punch‑Out!! editor.

This module defines a CLI entry point using `click`.  It exposes
subcommands for listing fighters, reading names/stats, and writing
modifications back to the ROM.  The CLI is minimal but provides a clean
foundation for further development.
"""

from __future__ import annotations

import click
from rich.console import Console
from rich.table import Table

from .rom import Rom
from .fighter import fighters


console = Console()


@click.group()
def cli() -> None:
    """Super Punch‑Out!! ROM editor.

    Use the available subcommands to inspect and modify your ROM.  Run
    ``supo COMMAND --help`` for details on a specific command.
    """


@cli.command(name="list-fighters")
@click.argument("rom_path", type=click.Path(exists=True))
@click.option(
    "--with-offsets",
    is_flag=True,
    help="Include file offsets for each fighter's name in the output table.",
)
def list_fighters(rom_path: str, with_offsets: bool) -> None:
    """Display the fighters and their current names.

    Reads the ROM at ``rom_path`` and prints each fighter's current name.  Use
    ``--with-offsets`` to include the file offset for each name, which can be
    helpful when reverse engineering the ROM.
    """
    rom = Rom.from_file(rom_path)
    table = Table(title="Fighters")
    table.add_column("Label")
    table.add_column("Name")
    if with_offsets:
        table.add_column("Name Offset", justify="right")
    for fighter in fighters:
        name = fighter.get_name(rom)
        if with_offsets:
            table.add_row(fighter.entry.label, name, hex(fighter.entry.name_offset))
        else:
            table.add_row(fighter.entry.label, name)
    console.print(table)


@cli.command(name="set-name")
@click.argument("rom_path", type=click.Path(exists=True))
@click.option(
    "--fighter",
    "fighter_label",
    required=True,
    help="Label of the fighter to modify (e.g. 'Gabby Jay').",
)
@click.option(
    "--name",
    "new_name",
    required=True,
    help="New name to assign to the fighter.  ASCII only; longer names will be truncated.",
)
@click.argument("output_path", type=click.Path(), required=False)
def set_name(rom_path: str, fighter_label: str, new_name: str, output_path: str | None) -> None:
    """Write a new name for a fighter and save the patched ROM.

    By default the ROM is modified in place; provide ``output_path`` to write
    changes to a new file instead.
    """
    rom = Rom.from_file(rom_path)
    target = None
    for fighter in fighters:
        if fighter.entry.label.lower() == fighter_label.lower():
            target = fighter
            break
    if target is None:
        console.print(f"[red]Error:[/red] fighter '{fighter_label}' not found.")
        return
    target.set_name(rom, new_name)
    dest = output_path or rom_path
    rom.save(dest)
    console.print(f"[green]Updated[/green] {target.entry.label} → {new_name} (saved to {dest})")


@cli.command(name="set-stats")
@click.argument("rom_path", type=click.Path(exists=True))
@click.option(
    "--fighter",
    "fighter_label",
    required=True,
    help="Label of the fighter to modify (e.g. 'Gabby Jay').",
)
@click.option(
    "--stats",
    "stats_hex",
    required=True,
    help="Raw stat bytes as a hex string (e.g. '0102030405060708').  Must be exactly the expected length.",
)
@click.argument("output_path", type=click.Path(), required=False)
def set_stats(rom_path: str, fighter_label: str, stats_hex: str, output_path: str | None) -> None:
    """Write a new raw stat block for a fighter.

    ``stats_hex`` should be a sequence of hexadecimal characters with no
    separators.  The length of the resulting bytes must match the
    ``stats_length`` defined for the fighter in :mod:`constants`.  For
    example, if a fighter's stats length is 8 bytes you should pass exactly
    16 hex characters.
    """
    # Remove optional 0x prefix and spaces
    hex_str = stats_hex.replace("0x", "").replace(" ", "")
    try:
        raw_bytes = bytes.fromhex(hex_str)
    except ValueError:
        console.print(f"[red]Error:[/red] '{stats_hex}' is not valid hexadecimal.")
        return
    rom = Rom.from_file(rom_path)
    target = None
    for fighter in fighters:
        if fighter.entry.label.lower() == fighter_label.lower():
            target = fighter
            break
    if target is None:
        console.print(f"[red]Error:[/red] fighter '{fighter_label}' not found.")
        return
    if len(raw_bytes) != target.entry.stats_length:
        console.print(
            f"[red]Error:[/red] stat block must be {target.entry.stats_length} bytes ({target.entry.stats_length*2} hex characters)"
        )
        return
    target.set_stats(rom, raw_bytes)
    dest = output_path or rom_path
    rom.save(dest)
    console.print(f"[green]Updated[/green] stats for {target.entry.label} (saved to {dest})")


@cli.command(name="dump-stats")
@click.argument("rom_path", type=click.Path(exists=True))
@click.option(
    "--fighter",
    "fighter_label",
    required=True,
    help="Label of the fighter whose stats to display.",
)
def dump_stats(rom_path: str, fighter_label: str) -> None:
    """Print the raw stat bytes for a given fighter as hex.

    Since the meaning of the stat block is not defined by this tool,
    this command simply dumps the bytes so you can analyze them elsewhere.
    """
    rom = Rom.from_file(rom_path)
    target = None
    for fighter in fighters:
        if fighter.entry.label.lower() == fighter_label.lower():
            target = fighter
            break
    if target is None:
        console.print(f"[red]Error:[/red] fighter '{fighter_label}' not found.")
        return
    raw = target.get_stats(rom)
    console.print(f"{target.entry.label} stats: {raw.hex()}")


if __name__ == "__main__":  # pragma: no cover
    cli()