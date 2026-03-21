#!/usr/bin/env python3
"""
Visual ROM difference map - optimized version
"""

def read_rom(path):
    with open(path, 'rb') as f:
        return f.read()

def visualize_diff(data1, data2, chunk_size=4096):
    """Create visual diff map"""
    total = len(data1)
    chunks = (total + chunk_size - 1) // chunk_size
    
    print(f"Chunk size: {chunk_size:,} bytes | Total chunks: {chunks}")
    print("Legend: . = identical, + = <10% different, * = <50% different, # = >50% different")
    print()
    
    line_width = 64  # 64 chunks = 256KB per line at 4KB chunks
    
    for row in range(0, chunks, line_width):
        line = ""
        for col in range(line_width):
            chunk = row + col
            if chunk >= chunks:
                break
            
            start = chunk * chunk_size
            end = min(start + chunk_size, total)
            
            diffs = sum(1 for i in range(start, end) if data1[i] != data2[i])
            pct = diffs / (end - start)
            
            if pct == 0:
                line += "."
            elif pct < 0.1:
                line += "+"
            elif pct < 0.5:
                line += "*"
            else:
                line += "#"
        
        start_addr = row * chunk_size
        print(f"0x{start_addr:06X}: {line}")

def analyze_regions(usa, eu, jp):
    """Analyze regions in detail"""
    print(f"\n{'='*60}")
    print("REGION ANALYSIS")
    print(f"{'='*60}")
    
    regions = [
        (0x000000, 0x040000, "0-256KB (Bank 0-1)"),
        (0x040000, 0x080000, "256-512KB (Bank 2-3)"),
        (0x080000, 0x0C0000, "512-768KB (Bank 4-5)"),
        (0x0C0000, 0x100000, "768KB-1MB (Bank 6-7)"),
        (0x100000, 0x140000, "1-1.25MB (Bank 8-9)"),
        (0x140000, 0x180000, "1.25-1.5MB (Bank 10-11)"),
        (0x180000, 0x1C0000, "1.5-1.75MB (Bank 12-13)"),
        (0x1C0000, 0x200000, "1.75-2MB (Bank 14-15)"),
    ]
    
    for start, end, desc in regions:
        size = end - start
        usa_eu = sum(1 for i in range(start, end) if usa[i] != eu[i])
        usa_jp = sum(1 for i in range(start, end) if usa[i] != jp[i])
        eu_jp = sum(1 for i in range(start, end) if eu[i] != jp[i])
        
        print(f"\n{desc}:")
        print(f"  USA vs EU: {100*usa_eu/size:5.1f}% ({usa_eu:,} bytes)")
        print(f"  USA vs JP: {100*usa_jp/size:5.1f}% ({usa_jp:,} bytes)")
        print(f"  EU vs JP:  {100*eu_jp/size:5.1f}% ({eu_jp:,} bytes)")

def find_largest_differences(usa, eu, jp, min_size=1024):
    """Find the largest contiguous difference regions"""
    print(f"\n{'='*60}")
    print("LARGEST DIFFERENCE REGIONS (USA vs EU)")
    print(f"{'='*60}")
    
    # Find USA vs EU diff regions
    in_diff = False
    diff_start = 0
    diff_ranges = []
    
    for i in range(len(usa)):
        if usa[i] != eu[i]:
            if not in_diff:
                diff_start = i
                in_diff = True
        else:
            if in_diff:
                if i - diff_start >= min_size:
                    diff_ranges.append((diff_start, i))
                in_diff = False
    
    if in_diff and len(usa) - diff_start >= min_size:
        diff_ranges.append((diff_start, len(usa)))
    
    # Sort by size
    diff_ranges.sort(key=lambda x: x[1] - x[0], reverse=True)
    
    print("Top 15 largest difference regions:")
    for start, end in diff_ranges[:15]:
        size = end - start
        print(f"  0x{start:06X} - 0x{end:06X} ({size:,} bytes)")
        
        # Check if same region differs in other comparisons
        usa_jp_diff = sum(1 for i in range(start, end) if usa[i] != jp[i])
        eu_jp_diff = sum(1 for i in range(start, end) if eu[i] != jp[i])
        print(f"    Also differs USA-JP: {100*usa_jp_diff/size:.0f}%, EU-JP: {100*eu_jp_diff/size:.0f}%")

def main():
    rom_dir = "."
    
    usa = read_rom(f"{rom_dir}/Super Punch-Out!! (USA).sfc")
    eu = read_rom(f"{rom_dir}/Super Punch-Out!! (Europe).sfc")
    jp = read_rom(f"{rom_dir}/Super Punch-Out!! (Japan) (NP).sfc")
    
    print("="*60)
    print("USA vs EUROPE DIFF MAP")
    print("="*60)
    visualize_diff(usa, eu)
    
    print("\n" + "="*60)
    print("USA vs JAPAN DIFF MAP")
    print("="*60)
    visualize_diff(usa, jp)
    
    print("\n" + "="*60)
    print("EUROPE vs JAPAN DIFF MAP")
    print("="*60)
    visualize_diff(eu, jp)
    
    analyze_regions(usa, eu, jp)
    find_largest_differences(usa, eu, jp)

if __name__ == '__main__':
    main()
