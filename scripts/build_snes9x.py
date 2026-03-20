#!/usr/bin/env python3
"""
Build Snes9x libretro core for the Super Punch-Out!! editor.
"""

import os
import sys
import subprocess
import platform
import shutil

SNES9X_REPO = "https://github.com/snes9xgit/snes9x.git"
BUILD_DIR = "build/snes9x"
OUTPUT_DIR = "apps/desktop/src-tauri/binaries"


def get_platform():
    """Get current platform."""
    system = platform.system().lower()
    if system == "windows":
        return "windows"
    elif system == "darwin":
        return "macos"
    else:
        return "linux"


def get_lib_extension():
    """Get library extension for current platform."""
    system = get_platform()
    if system == "windows":
        return ".dll"
    elif system == "macos":
        return ".dylib"
    else:
        return ".so"


def clone_snes9x():
    """Clone Snes9x repository."""
    if os.path.exists(BUILD_DIR):
        print("Snes9x already cloned, updating...")
        subprocess.run(["git", "pull"], cwd=BUILD_DIR, check=True)
    else:
        print("Cloning Snes9x...")
        os.makedirs(os.path.dirname(BUILD_DIR), exist_ok=True)
        subprocess.run(
            ["git", "clone", "--depth", "1", SNES9X_REPO, BUILD_DIR],
            check=True
        )


def build_snes9x():
    """Build Snes9x libretro core."""
    libretro_dir = os.path.join(BUILD_DIR, "libretro")
    
    print(f"Building Snes9x for {get_platform()}...")
    
    # Clean previous build
    subprocess.run(["make", "-f", "Makefile", "clean"], cwd=libretro_dir)
    
    # Build
    result = subprocess.run(
        ["make", "-f", "Makefile", "-j4"],
        cwd=libretro_dir,
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print("Build failed!")
        print(result.stderr)
        sys.exit(1)
    
    print("Build successful!")


def copy_library():
    """Copy built library to output directory."""
    libretro_dir = os.path.join(BUILD_DIR, "libretro")
    ext = get_lib_extension()
    lib_name = f"snes9x_libretro{ext}"
    
    src = os.path.join(libretro_dir, lib_name)
    dst_dir = OUTPUT_DIR
    dst = os.path.join(dst_dir, lib_name)
    
    if not os.path.exists(src):
        print(f"Library not found: {src}")
        print("Contents of libretro directory:")
        for f in os.listdir(libretro_dir):
            print(f"  {f}")
        sys.exit(1)
    
    os.makedirs(dst_dir, exist_ok=True)
    shutil.copy2(src, dst)
    print(f"Copied {lib_name} to {dst_dir}")


def main():
    """Main build process."""
    print("=" * 60)
    print("Building Snes9x libretro core")
    print("=" * 60)
    
    try:
        clone_snes9x()
        build_snes9x()
        copy_library()
        
        print("\n" + "=" * 60)
        print("Build complete!")
        print("=" * 60)
        print(f"\nThe emulator core is ready at:")
        print(f"  {OUTPUT_DIR}/snes9x_libretro{get_lib_extension()}")
        print("\nYou can now run the editor with embedded emulator support.")
        
    except subprocess.CalledProcessError as e:
        print(f"Command failed: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
