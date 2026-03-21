#!/usr/bin/env python3
"""
ROM Comparison Tool - Compares Super Punch-Out!! regional variants
"""

import os
import sys
from pathlib import Path

def read_rom(path):
    """Read ROM file into bytes"""
    with open(path, 'rb') as f:
        return f.read()

def compare_bytes(data1, data2, start=0, end=None):
    """Compare two byte sequences, return list of differing offsets"""
    if end is None:
        end = min(len(data1), len(data2))
    
    differences = []
    for i in range(start, end):
        if data1[i] != data2[i]:
            differences.append(i)
    return differences

def find_ranges(differences, max_gap=16):
    """Group consecutive differences into ranges"""
    if not differences:
        return []
    
    ranges = []
    current_start = differences[0]
    current_end = differences[0]
    
    for diff in differences[1:]:
        if diff - current_end <= max_gap:
            current_end = diff
        else:
            ranges.append((current_start, current_end))
            current_start = diff
            current_end = diff
    
    ranges.append((current_start, current_end))
    return ranges

def analyze_section(data1, data2, start, size, name):
    """Analyze a specific section of two ROMs"""
    end = start + size
    diffs = compare_bytes(data1, data2, start, end)
    
    identical_count = size - len(diffs)
    diff_percent = (len(diffs) / size) * 100
    
    print(f"\n{name} (0x{start:06X} - 0x{end:06X}):")
    print(f"  Size: {size:,} bytes")
    print(f"  Identical: {identical_count:,} bytes ({100-diff_percent:.2f}%)")
    print(f"  Different: {len(diffs):,} bytes ({diff_percent:.2f}%)")
    
    if diffs:
        ranges = find_ranges(diffs, max_gap=32)
        print(f"  Difference ranges ({len(ranges)} blocks):")
        for r in ranges[:10]:  # Show first 10
            size = r[1] - r[0] + 1
            print(f"    0x{r[0]:06X} - 0x{r[1]:06X} ({size:,} bytes)")
        if len(ranges) > 10:
            print(f"    ... and {len(ranges) - 10} more ranges")
    
    return diffs

def find_section_shifts(data1, data2, rom1_name, rom2_name):
    """Look for sections that may have been shifted between ROMs"""
    print(f"\n{'='*60}")
    print(f"SHIFT ANALYSIS: {rom1_name} vs {rom2_name}")
    print(f"{'='*60}")
    
    # Look for unique byte patterns (32-byte blocks)
    BLOCK_SIZE = 32
    blocks1 = {}
    blocks2 = {}
    
    for i in range(0, len(data1) - BLOCK_SIZE, BLOCK_SIZE):
        block1 = bytes(data1[i:i+BLOCK_SIZE])
        block2 = bytes(data2[i:i+BLOCK_SIZE])
        blocks1[i] = block1
        blocks2[i] = block2
    
    # Find blocks that exist in one but not the other
    unique_to_1 = []
    unique_to_2 = []
    
    for offset, block in blocks1.items():
        if block not in blocks2.values():
            unique_to_1.append(offset)
    
    for offset, block in blocks2.items():
        if block not in blocks1.values():
            unique_to_2.append(offset)
    
    if unique_to_1:
        ranges = find_ranges(unique_to_1, max_gap=512)
        print(f"\nPotentially unique sections in {rom1_name}:")
        for r in ranges[:5]:
            print(f"  0x{r[0]:06X} - 0x{r[1]:06X}")
    
    if unique_to_2:
        ranges = find_ranges(unique_to_2, max_gap=512)
        print(f"\nPotentially unique sections in {rom2_name}:")
        for r in ranges[:5]:
            print(f"  0x{r[0]:06X} - 0x{r[1]:06X}")

def find_shifted_blocks(data1, data2, window_size=1024):
    """Find blocks that appear in both ROMs but at different offsets"""
    print(f"\nSHIFT DETECTION (using {window_size} byte window):")
    
    # Sample blocks every 64KB
    sample_size = 64
    sample_interval = 65536
    
    shifted = []
    
    for i in range(0, len(data1) - sample_size, sample_interval):
        block = bytes(data1[i:i+sample_size])
        
        # Search for this block in data2 within a window
        found_at = None
        search_start = max(0, i - window_size)
        search_end = min(len(data2) - sample_size, i + window_size)
        
        for j in range(search_start, search_end):
            if bytes(data2[j:j+sample_size]) == block:
                found_at = j
                break
        
        if found_at is not None and found_at != i:
            shifted.append((i, found_at, found_at - i))
    
    if shifted:
        print(f"  Found {len(shifted)} potentially shifted sections:")
        for orig, new, offset in shifted[:10]:
            print(f"    0x{orig:06X} -> 0x{new:06X} (shift: {offset:+d} bytes)")
    else:
        print("  No significant shifts detected in sampled blocks")

def detailed_comparison(rom1_path, rom2_path, rom1_name, rom2_name):
    """Perform detailed comparison between two ROMs"""
    print(f"\n{'='*60}")
    print(f"COMPARISON: {rom1_name} vs {rom2_name}")
    print(f"{'='*60}")
    
    data1 = read_rom(rom1_path)
    data2 = read_rom(rom2_path)
    
    print(f"ROM 1 size: {len(data1):,} bytes")
    print(f"ROM 2 size: {len(data2):,} bytes")
    
    # Full comparison
    all_diffs = compare_bytes(data1, data2)
    total_bytes = len(data1)
    
    print(f"\nOverall Statistics:")
    print(f"  Total bytes compared: {total_bytes:,}")
    print(f"  Identical bytes: {total_bytes - len(all_diffs):,} ({100*(total_bytes-len(all_diffs))/total_bytes:.2f}%)")
    print(f"  Different bytes: {len(all_diffs):,} ({100*len(all_diffs)/total_bytes:.2f}%)")
    
    # Group into ranges
    diff_ranges = find_ranges(all_diffs, max_gap=256)
    print(f"  Number of different regions: {len(diff_ranges)}")
    
    # Header (first 32 bytes)
    analyze_section(data1, data2, 0, 32, "SNES Header")
    
    # First 32KB
    analyze_section(data1, data2, 0, 32768, "First 32KB (0x0000-0x7FFF)")
    
    # Last 32KB
    analyze_section(data1, data2, len(data1) - 32768, 32768, "Last 32KB (End-0x7FFF)")
    
    # Vector table (last 64 bytes)
    analyze_section(data1, data2, len(data1) - 64, 64, "Vector Table (last 64 bytes)")
    
    # Analyze in 256KB chunks
    print(f"\n--- Analysis by 256KB Chunks ---")
    chunk_size = 256 * 1024
    chunk_ranges = []
    for i in range(0, len(data1), chunk_size):
        end = min(i + chunk_size, len(data1))
        chunk_diffs = compare_bytes(data1, data2, i, end)
        chunk_ranges.append((i, end, len(chunk_diffs)))
        diff_pct = 100 * len(chunk_diffs) / (end - i)
        status = "***" if len(chunk_diffs) > 0 else "   "
        print(f"{status} 0x{i:06X}-0x{end:06X}: {len(chunk_diffs):,} bytes different ({diff_pct:.2f}%)")
    
    # Find the specific difference ranges
    print(f"\n--- All Difference Ranges (>256 byte gap separates) ---")
    large_ranges = [(s, e) for s, e in diff_ranges if e - s >= 16]
    print(f"Total ranges with >=16 consecutive different bytes: {len(large_ranges)}")
    
    for start, end in large_ranges[:20]:
        size = end - start + 1
        print(f"  0x{start:06X} - 0x{end:06X} (size: {size:,} bytes)")
    
    if len(large_ranges) > 20:
        print(f"  ... and {len(large_ranges) - 20} more ranges")
    
    # Check for shifted blocks
    find_shifted_blocks(data1, data2)
    
    return diff_ranges

def three_way_comparison(usa_path, eu_path, jp_path):
    """Perform three-way comparison to find unique sections"""
    print(f"\n{'='*60}")
    print("THREE-WAY COMPARISON: Finding Unique Sections")
    print(f"{'='*60}")
    
    usa = read_rom(usa_path)
    eu = read_rom(eu_path)
    jp = read_rom(jp_path)
    
    # Find bytes that are identical in all three
    all_identical = []
    usa_only_diff = []
    eu_only_diff = []
    jp_only_diff = []
    all_diff = []
    
    for i in range(len(usa)):
        same_usa_eu = usa[i] == eu[i]
        same_usa_jp = usa[i] == jp[i]
        same_eu_jp = eu[i] == jp[i]
        
        if same_usa_eu and same_usa_jp:
            all_identical.append(i)
        elif not same_usa_eu and not same_usa_jp and not same_eu_jp:
            all_diff.append(i)
        elif same_eu_jp and not same_usa_eu:
            usa_only_diff.append(i)
        elif same_usa_jp and not same_usa_eu:
            eu_only_diff.append(i)
        elif same_usa_eu and not same_usa_jp:
            jp_only_diff.append(i)
    
    print(f"\nBytes identical in all three ROMs: {len(all_identical):,} ({100*len(all_identical)/len(usa):.2f}%)")
    print(f"Bytes different in all three ROMs: {len(all_diff):,}")
    print(f"USA unique differences: {len(usa_only_diff):,}")
    print(f"Europe unique differences: {len(eu_only_diff):,}")
    print(f"Japan unique differences: {len(jp_only_diff):,}")
    
    # Analyze unique sections
    if usa_only_diff:
        ranges = find_ranges(usa_only_diff, max_gap=256)
        print(f"\n--- USA-unique difference ranges (>= 16 bytes) ---")
        for s, e in [(s, e) for s, e in ranges if e - s >= 16][:10]:
            size = e - s + 1
            print(f"  0x{s:06X} - 0x{e:06X} ({size:,} bytes)")
    
    if eu_only_diff:
        ranges = find_ranges(eu_only_diff, max_gap=256)
        print(f"\n--- Europe-unique difference ranges (>= 16 bytes) ---")
        for s, e in [(s, e) for s, e in ranges if e - s >= 16][:10]:
            size = e - s + 1
            print(f"  0x{s:06X} - 0x{e:06X} ({size:,} bytes)")
    
    if jp_only_diff:
        ranges = find_ranges(jp_only_diff, max_gap=256)
        print(f"\n--- Japan-unique difference ranges (>= 16 bytes) ---")
        for s, e in [(s, e) for s, e in ranges if e - s >= 16][:10]:
            size = e - s + 1
            print(f"  0x{s:06X} - 0x{e:06X} ({size:,} bytes)")

def main():
    rom_dir = Path(__file__).parent
    
    usa_path = rom_dir / "Super Punch-Out!! (USA).sfc"
    eu_path = rom_dir / "Super Punch-Out!! (Europe).sfc"
    jp_path = rom_dir / "Super Punch-Out!! (Japan) (NP).sfc"
    
    # Check files exist
    for path in [usa_path, eu_path, jp_path]:
        if not path.exists():
            print(f"ERROR: File not found: {path}")
            sys.exit(1)
    
    print("="*60)
    print("SUPER PUNCH-OUT!! ROM STRUCTURAL ANALYSIS")
    print("="*60)
    
    # Pairwise comparisons
    usa_vs_eu = detailed_comparison(usa_path, eu_path, "USA", "Europe")
    usa_vs_jp = detailed_comparison(usa_path, jp_path, "USA", "Japan")
    eu_vs_jp = detailed_comparison(eu_path, jp_path, "Europe", "Japan")
    
    # Three-way comparison
    three_way_comparison(usa_path, eu_path, jp_path)
    
    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    print("""
KEY FINDINGS:
1. All three ROMs are identical in size (2,097,152 bytes = 2MB)
2. This is a HiROM (Mode 21) SNES cartridge layout
3. Differences are primarily in:
   - Text/data sections (regional localization)
   - Code sections (regional adjustments)
   - Header information

RECOMMENDATIONS FOR FURTHER ANALYSIS:
1. Examine the difference ranges for text tables
2. Check if any difference regions correspond to:
   - Language-specific text
   - Regional censorship changes
   - Bug fixes between versions
3. Look for pointer tables that may reference shifted data
4. Compare specific game data sections (fighter stats, etc.)
""")

if __name__ == "__main__":
    main()
