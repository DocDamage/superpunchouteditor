# Super Punch‑Out!! Editor

This project is a starting point for developing a **Super Punch‑Out!!** ROM editor.  It is built with Python and aims to make it straightforward to inspect and modify data within the Super Nintendo ROM for *Super Punch‑Out!!*.  Right now the code focuses on providing a clean foundation—you can load a ROM, view and edit high‑level fighter attributes, and write the modified ROM back to disk.  As you become familiar with the game’s data structures you can extend this framework to cover additional assets such as graphics, palettes, and AI logic.

> **Disclaimer**: This editor does not ship with any copyrighted game data.  You must supply your own *Super Punch‑Out!!* ROM (commonly in `.sfc` or `.smc` format) to use it.  Use of ROM images is subject to your local laws and the rights of the copyright holder.

## Features

* Load a Super Punch‑Out!! ROM into memory for inspection and modification.
* Define fighter entries with configurable offsets and lengths.  By default the names and stats are placeholders—you are expected to update the offsets once you identify the correct locations in the ROM.
* Read and write fighter names via a simple command line interface.
* Patch the ROM in place or write out a modified copy.

## Getting started

### Installation

1. Install a recent Python (3.10 or newer is recommended).
2. Install dependencies:

   ```sh
   pip install -r requirements.txt
   ```

3. (Optional) Install the package in editable mode so command line tools are available:

   ```sh
   pip install -e .
   ```

### Usage

After installation you should have a `supo` command available.  The following examples assume you have obtained a legitimate copy of the *Super Punch‑Out!!* ROM.

List the fighters and their current names:

```sh
python supo.py list-fighters path/to/super_punch_out.sfc
```

Change Gabby Jay’s name in the ROM:

```sh
python supo.py set-name --fighter "Gabby Jay" --name "Punchy McFace" path/to/super_punch_out.sfc patched.sfc
```

If you omit the output file, the original ROM will be modified in place.

### Updating offsets

The default offsets and lengths for fighters are placeholders.  You will need to inspect the ROM and determine where the relevant strings and stats are stored.  Once you know the correct addresses, update the values in `superpunchouteditor/constants.py` accordingly.  Offsets are specified relative to the start of the ROM (i.e. file offsets, not SNES memory addresses).  You may find tools like debuggers, disassemblers, or existing documentation helpful in locating these structures.

## Project structure

```
superpunchouteditor/
├── README.md           – This file
├── pyproject.toml       – Package metadata
├── requirements.txt     – External dependencies
├── supo.py              – Command‑line entry point
├── superpunchouteditor/  – Python package
│   ├── __init__.py
│   ├── constants.py     – Default offsets and lengths for data tables
│   ├── rom.py           – ROM loading and low‑level read/write utilities
│   ├── fighter.py       – Fighter model and serialization logic
│   └── editor.py        – High‑level operations and CLI hooks
└── tests/
    └── test_rom.py      – Unit tests for basic ROM operations
```

## Contributing

1. Fork this repository and create your branch from `main`.
2. If you discover the correct offsets or implement additional features (stats editing, graphic editing, etc.), please add tests to cover them.
3. Open a pull request describing your changes.

### Coding style

This project uses [black](https://black.readthedocs.io/en/stable/) for formatting and [flake8](https://flake8.pycqa.org/en/latest/) for linting.  Install the development dependencies listed in `requirements.txt` if you plan to contribute.  Tests are run with `pytest`.

## License

This project is released under the MIT License.  See the `LICENSE` file for details.  **Note:** This license applies only to the code in this repository; it does not grant any rights to the *Super Punch‑Out!!* ROM itself.