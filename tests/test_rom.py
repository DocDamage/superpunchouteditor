import sys
import pathlib

# Add the package root to sys.path so imports work when running tests without installation.
PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT))

from superpunchouteditor.rom import Rom


def test_get_set_bytes(tmp_path):
    # Create a dummy ROM file
    original = bytes(range(256))
    rom_path = tmp_path / "dummy.sfc"
    rom_path.write_bytes(original)

    # Load ROM
    rom = Rom.from_file(str(rom_path))

    # Read 10 bytes at offset 100
    chunk = rom.get_bytes(100, 10)
    assert chunk == original[100:110]

    # Modify bytes 0x10–0x19
    new_data = b"abcdefghij"
    rom.set_bytes(0x10, new_data)
    assert rom.get_bytes(0x10, len(new_data)) == new_data

    # Save and reload
    patched_path = tmp_path / "patched.sfc"
    rom.save(str(patched_path))
    patched = patched_path.read_bytes()
    assert patched[0x10 : 0x10 + len(new_data)] == new_data