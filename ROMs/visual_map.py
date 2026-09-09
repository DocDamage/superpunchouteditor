#!/usr/bin/env python3
"""
Visual ROM difference map and exclusive content finder
"""

import os
from pathlib import Path

def read_rom(path):
    with open(path, 'rb') as f:
        return f.read()

def create_diff_map(data1, data2, chunk_size=1024):
    """Create a visual diff map using chunk_size byte chunks"""
    chunks = (len(data1) + chunk_size - 1) // chunk_size
    diff_map = []
    
    for i in range(chunks):
        start = i * chunk_size
        end = min(start + chunk_size, len(data1))
        
        chunk1 = data1[start:end]
        chunk2 = data2[start:end]
        
        if chunk1 != chunk2:
            # Count differences in this chunk
            diffs = sum(1 for a, b in zip(chunk1, chunk2) if a != b)
            diff_map.append((i, start, end, diffs, len(chunk1)))
    
    return diff_map

def visualize_diff_map(diff_map, total_chunks, chunk_size, title):
    """Create ASCII visualization of differences"""
    print(f"\n{'='*60}")
    print(f"{title}")
    print(f"{'='*60}")
    print(f"Chunk size: {chunk_size:,} bytes | Total chunks: {total_chunks}")
    print(f"Legend: . = identical, ! = different, # = mostly different (>50%)")
    print()
    
    # Create visual representation (64 chars per line = 64KB per line at 1KB chunks)
    line_width = 64
    
    diff_set = {d[0] for d in diff_map}
    
    for row in range(0, total_chunks, line_width):
        line = ""
        for col in range(line_width):
            chunk_idx = row + col
            if chunk_idx >= total_chunks:
                break
            
            if chunk_idx in diff_set:
                # Find the chunk info
                for info in diff_map:
                    if info[0] == chunk_idx:
                        _, _, _, diffs, size = info
                        pct = diffs / size
                        if pct > 0.5:
                            line += "#"
                        else:
                            line += "!"
                        break
            else:
                line += "."
        
        start_addr = row * chunk_size
        end_addr = min((row + line_width) * chunk_size, total_chunks * chunk_size) - 1
        print(f"0x{start_addr:06X}: {line}")

def find_exclusive_sections(usa_data, eu_data, jp_data, min_size=256):
    """Find sections that exist in one ROM but not others"""
    print(f"\n{'='*60}")
    print("EXCLUSIVE CONTENT ANALYSIS")
    print(f"{'='*60}")
    print(f"Looking for unique blocks >= {min_size} bytes...")
    print()
    
    # Sample at 64-byte intervals
    sample = 64
    
    usa_blocks = {}
    eu_blocks = {}
    jp_blocks = {}
    
    for i in range(0, len(usa_data) - sample + 1, sample):
        usa_blocks[i] = bytes(usa_data[i:i+sample])
        eu_blocks[i] = bytes(eu_data[i:i+sample])
        jp_blocks[i] = bytes(jp_data[i:i+sample])
    
    # Find truly unique blocks (not in either other ROM)
    usa_unique = []
    eu_unique = []
    jp_unique = []
    
    for offset in usa_blocks:
        block = usa_blocks[offset]
        in_eu = block in eu_blocks.values()
        in_jp = block in jp_blocks.values()
        
        if not in_eu and not in_jp:
            usa_unique.append(offset)
    
    for offset in eu_blocks:
        block = eu_blocks[offset]
        in_usa = block in usa_blocks.values()
        in_jp = block in jp_blocks.values()
        
        if not in_usa and not in_jp:
            eu_unique.append(offset)
    
    for offset in jp_blocks:
        block = jp_blocks[offset]
        in_usa = block in usa_blocks.values()
        in_eu = block in eu_blocks.values()
        
        if not in_usa and not in_eu:
            jp_unique.append(offset)
    
    def group_ranges(offsets, interval):
        if not offsets:
            return []
        ranges = []
        start = offsets[0]
        end = offsets[0]
        
        for off in offsets[1:]:
            if off - end <= interval * 2:
                end = off
            else:
                ranges.append((start, end + interval))
                start = off
                end = off
        ranges.append((start, end + interval))
        return ranges
    
    usa_ranges = group_ranges(usa_unique, sample)
    eu_ranges = group_ranges(eu_unique, sample)
    jp_ranges = group_ranges(jp_unique, sample)
    
    # Filter by minimum size
    usa_ranges = [(s, e) for s, e in usa_ranges if e - s >= min_size]
    eu_ranges = [(s, e) for s, e in eu_ranges if e - s >= min_size]
    jp_ranges = [(s, e) for s, e in jp_ranges if e - s >= min_size]
    
    print(f"USA exclusive regions: {len(usa_ranges)}")
    for s, e in usa_ranges[:10]:
        size = e - s
        print(f"  0x{s:06X} - 0x{e:06X} ({size:,} bytes)")
    
    print(f"\nEurope exclusive regions: {len(eu_ranges)}")
    for s, e in eu_ranges[:10]:
        size = e - s
        print(f"  0x{s:06X} - 0x{e:06X} ({size:,} bytes)")
    
    print(f"\nJapan exclusive regions: {len(jp_ranges)}")
    for s, e in jp_ranges[:10]:
        size = e - s
        print(f"  0x{s:06X} - 0x{e:06X} ({size:,} bytes)")

def analyze_text_regions(data1, data2, start, size, name):
    """Analyze potential text regions"""
    print(f"\n--- {name} (0x{start:06X}) ---")
    
    printable1 = sum(1 for b in data1[start:start+size] if 32 <= b < 127 or b in (0x0A, 0x0D))
    printable2 = sum(1 for b in data2[start:start+size] if 32 <= b < 127 or b in (0x0A, 0x0D))
    
    print(f"ROM 1 printable ratio: {100*printable1/size:.1f}%")
    print(f"ROM 2 printable ratio: {100*printable2/size:.1f}%")
    
    if printable1 > size * 0.7 or printable2 > size * 0.7:
        print("  -> Likely TEXT region")

def main():
    rom_dir = Path(__file__).parent
    
    usa_path = rom_dir / "Super Punch-Out!! (USA).sfc"
    eu_path = rom_dir / "Super Punch-Out!! (Europe).sfc"
    jp_path = rom_dir / "Super Punch-Out!! (Japan) (NP).sfc"
    
    usa = read_rom(usa_path)
    eu = read_rom(eu_path)
    jp = read_rom(jp_path)
    
    # Visual diff maps
    chunk_size = 2048  # 2KB chunks
    total_chunks = (len(usa) + chunk_size - 1) // chunk_size
    
    diff_usa_eu = create_diff_map(usa, eu, chunk_size)
    visualize_diff_map(diff_usa_eu, total_chunks, chunk_size, "USA vs Europe Diff Map")
    
    diff_usa_jp = create_diff_map(usa, jp, chunk_size)
    visualize_diff_map(diff_usa_jp, total_chunks, chunk_size, "USA vs Japan Diff Map")
    
    diff_eu_jp = create_diff_map(eu, jp, chunk_size)
    visualize_diff_map(diff_eu_jp, total_chunks, chunk_size, "Europe vs Japan Diff Map")
    
    # Find exclusive sections
    find_exclusive_sections(usa, eu, jp, min_size=512)
    
    # Identify major difference regions
    print(f"\n{'='*60}")
    print("MAJOR DIFFERENCE REGIONS SUMMARY")
    print(f"{'='*60}")
    
    regions = [
        (0x000000, 0x080000, "First 512KB (Code/Data Bank 0-3)"),
        (0x080000, 0x100000, "512KB-1MB (Data Bank 4-7)"),
        (0x100000, 0x180000, "1MB-1.5MB (Data Bank 8-11)"),
        (0x180000, 0x200000, "1.5MB-2MB (Data Bank 12-15)"),
    ]
    
    for start, end, desc in regions:
        usa_eu_diffs = sum(1 for i in range(start, end) if usa[i] != eu[i])
        usa_jp_diffs = sum(1 for i in range(start, end) if usa[i] != jp[i])
        eu_jp_diffs = sum(1 for i in range(start, end) if eu[i] != jp[i])
        
        size = end - start
        
        print(f"\n{desc}:")
        print(f"  USA vs EU:  {100*usa_eu_diffs/size:6.2f}% different ({usa_eu_diffs:,} bytes)")
        print(f"  USA vs JP:  {100*usa_jp_diffs/size:6.2f}% different ({usa_jp_diffs:,} bytes)")
        print(f"  EU vs JP:   {100*eu_jp_diffs/size:6.2f}% different ({eu_jp_diffs:,} bytes)")

if __name__ == '__main__':
    main()
