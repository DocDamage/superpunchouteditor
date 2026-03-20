# Building Snes9x libretro Core

## Prerequisites

### Windows
- MSYS2 or Visual Studio 2019+
- Git

### macOS
- Xcode Command Line Tools
- Git

### Linux
- GCC or Clang
- SDL2 development libraries
- Git

## Build Steps

### 1. Clone Snes9x

```bash
git clone https://github.com/snes9xgit/snes9x.git
cd snes9x
```

### 2. Build libretro core

#### Windows (MSYS2)
```bash
cd libretro
make -f Makefile
# Output: snes9x_libretro.dll
```

#### macOS
```bash
cd libretro
make -f Makefile
# Output: snes9x_libretro.dylib
```

#### Linux
```bash
cd libretro
make -f Makefile
# Output: snes9x_libretro.so
```

### 3. Copy to Editor

Copy the built library to:
```
apps/desktop/src-tauri/binaries/
  snes9x_libretro.dll      (Windows)
  snes9x_libretro.dylib    (macOS)
  snes9x_libretro.so       (Linux)
```

## Troubleshooting

### Missing SDL2
Install SDL2 development libraries for your platform.

### Linker errors on Windows
Make sure you're using MSYS2 MinGW64 shell, not regular Command Prompt.

### macOS signing issues
You may need to codesign the dylib:
```bash
codesign -s "-" snes9x_libretro.dylib
```
