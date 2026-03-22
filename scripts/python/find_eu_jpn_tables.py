#!/usr/bin/env python3
"""
Targeted search for EU and JPN specific table addresses.
"""

import os
import struct
from typing import List, Tuple, Dict, Optional

ROM_PATHS = {
    'USA': 'ROMs/Super Punch-Out!! (USA).sfc',
    'EU': 'ROMs/Super Punch-Out!! (Europe).sfc',
    'JPN': 'ROMs/Super Punch-Out!! (Japan) (NP).sfc'
}

def pc_to_snes(pc_addr: int) -> str:
    bank = (pc_addr // 0x8000) | 0x80
    offset = (pc_addr % 0x8000) | 0x8000
    return f"${bank:02X}:{offset:04X}"

def snes_to_pc(bank: int, offset: int) -> int:
    return ((bank & 0x7F) * 0x8000) + (offset & 0x7FFF)

def read_bytes(rom_path: str, addr: int, size: int) -> bytes:
    with open(rom_path, 'rb') as f:
        f.seek(addr)
        return f.read(size)

def find_pattern_all(rom_data: bytes, pattern: bytes, start: int = 0, end: Optional[int] = None) -> List[int]:
    """Find all occurrences of a pattern"""
    if end is None:
        end = len(rom_data)
    results = []
    idx = rom_data.find(pattern, start, end)
    while idx != -1:
        results.append(idx)
        idx = rom_data.find(pattern, idx + 1, end)
    return results

def is_valid_bgr555_color(data: bytes, offset: int) -> bool:
    """Check if bytes at offset form a valid BGR555 color"""
    if offset + 1 >= len(data):
        return False
    color = data[offset] | (data[offset + 1] << 8)
    return color < 0x8000  # No priority bit set in palette data

def find_palette_tables(rom_type: str, rom_data: bytes, usa_rom: bytes) -> Dict:
    """Find palette table locations by searching for known patterns"""
    print(f"\n{'='*60}")
    print(f"PALETTE TABLE SEARCH - {rom_type}")
    print(f"{'='*60}")
    
    # USA palette addresses
    usa_palettes = [
        ("Gabby Jay", 0x06B9DA, 96),
        ("Bear Hugger", 0x06BC3C, 96),
        ("Piston Hurricane", 0x06BE9E, 96),
        ("Bald Bull", 0x06C100, 128),
        ("Bob Charlie", 0x06C382, 96),
        ("Dragon Chan", 0x06C5E4, 96),
        ("Masked Muscle", 0x06C846, 128),
        ("Mr. Sandman", 0x06CA88, 128),
        ("Aran Ryan", 0x06CCCA, 96),
        ("Heike Kagero", 0x06CF2C, 96),
        ("Mad Clown", 0x06D18E, 128),
        ("Super Macho Man", 0x06D3F0, 96),
        ("Narcis Prince", 0x06D652, 96),
        ("Hoy Quarlow", 0x06D8B4, 96),
        ("Rick Bruiser", 0x06DB16, 128),
        ("Nick Bruiser", 0x06DD58, 96),
    ]
    
    findings = {}
    
    # Search for each palette
    for name, usa_addr, size in usa_palettes:
        # Get USA palette data
        usa_palette = usa_rom[usa_addr:usa_addr+size]
        
        # Search for this pattern in target ROM
        # Use first 32 bytes as search pattern for uniqueness
        pattern = usa_palette[:32]
        matches = find_pattern_all(rom_data, pattern)
        
        if matches:
            # Found exact match
            addr = matches[0]
            snes = pc_to_snes(addr)
            offset = addr - usa_addr
            findings[name] = {
                'address': addr,
                'snes': snes,
                'offset_from_usa': offset,
                'confidence': 'HIGH' if len(matches) == 1 else 'MEDIUM'
            }
            print(f"{name:<20} PC 0x{addr:06X} {snes:<12} offset={offset:+d}")
        else:
            # Try searching for partial pattern (first 16 bytes)
            pattern = usa_palette[:16]
            matches = find_pattern_all(rom_data, pattern)
            
            if matches:
                addr = matches[0]
                snes = pc_to_snes(addr)
                offset = addr - usa_addr
                findings[name] = {
                    'address': addr,
                    'snes': snes,
                    'offset_from_usa': offset,
                    'confidence': 'MEDIUM'
                }
                print(f"{name:<20} PC 0x{addr:06X} {snes:<12} offset={offset:+d} (partial match)")
            else:
                print(f"{name:<20} NOT FOUND")
    
    return findings

def find_icon_tables(rom_type: str, rom_data: bytes, usa_rom: bytes) -> Dict:
    """Find icon graphics locations"""
    print(f"\n{'='*60}")
    print(f"ICON TABLE SEARCH - {rom_type}")
    print(f"{'='*60}")
    
    # USA icon addresses (512 bytes each)
    usa_icons = [
        ("Gabby Jay", 0x06B7D8),
        ("Bear Hugger", 0x06BA3A),
        ("Piston Hurricane", 0x06BC9C),
        ("Bald Bull", 0x06BEFE),
        ("Bob Charlie", 0x06C180),
    ]
    
    findings = {}
    
    for name, usa_addr in usa_icons[:3]:  # Just check first 3
        usa_icon = usa_rom[usa_addr:usa_addr+512]
        pattern = usa_icon[:64]  # Use first 64 bytes
        
        matches = find_pattern_all(rom_data, pattern)
        
        if matches:
            addr = matches[0]
            snes = pc_to_snes(addr)
            offset = addr - usa_addr
            findings[name] = {
                'address': addr,
                'snes': snes,
                'offset': offset
            }
            print(f"{name:<20} PC 0x{addr:06X} {snes:<12} offset={offset:+d}")
        else:
            print(f"{name:<20} NOT FOUND")
    
    return findings

def find_circuit_tables(rom_type: str, rom_data: bytes) -> Dict:
    """Find circuit assignment and related tables"""
    print(f"\n{'='*60}")
    print(f"CIRCUIT TABLE SEARCH - {rom_type}")
    print(f"{'='*60}")
    
    findings = {}
    
    # Circuit pattern: Minor (0,1,2,3), Major (4,5,6,7), World (8,9,10,11), Special (12,13,14,15)
    # Look for these byte sequences
    circuit_patterns = [
        (bytes([0, 1, 2, 3]), "Minor Circuit"),
        (bytes([4, 5, 6, 7]), "Major Circuit"),
        (bytes([8, 9, 10, 11]), "World Circuit"),
        (bytes([12, 13, 14, 15]), "Special Circuit"),
    ]
    
    for pattern, name in circuit_patterns:
        matches = find_pattern_all(rom_data, pattern)
        
        if matches:
            print(f"\n{name} pattern found at:")
            for addr in matches[:3]:
                snes = pc_to_snes(addr)
                context = rom_data[max(0, addr-8):addr+24]
                print(f"  PC 0x{addr:06X} {snes}")
                print(f"    Context: {context.hex()}")
    
    return findings

def find_text_tables(rom_type: str, rom_data: bytes) -> Dict:
    """Find text tables for names, intros, quotes"""
    print(f"\n{'='*60}")
    print(f"TEXT TABLE SEARCH - {rom_type}")
    print(f"{'='*60}")
    
    findings = {}
    
    # Search for boxer names using different encodings
    name_searches = []
    
    if rom_type == 'JPN':
        # Japanese encoding - look for katakana representations
        # Boxer names in Japanese would use different encoding
        print("Searching for Japanese text patterns...")
        
        # Look for patterns that might be Japanese text
        # Japanese text typically has high-byte characters
        for i in range(0x60000, 0x70000, 0x100):
            chunk = rom_data[i:i+0x100]
            high_bytes = sum(1 for b in chunk if 0x80 <= b <= 0xFF)
            if high_bytes > 128:  # More than half are high bytes
                preview = chunk[:32]
                print(f"  Potential text area at PC 0x{i:06X}: {preview.hex()}")
    else:
        # EU might have different text encoding or same as USA
        print("EU ROM - checking for text patterns...")
    
    return findings

def find_ai_script_tables(rom_type: str, rom_data: bytes, usa_rom: bytes) -> Dict:
    """Find AI script table locations"""
    print(f"\n{'='*60}")
    print(f"AI SCRIPT TABLE SEARCH - {rom_type}")
    print(f"{'='*60}")
    
    findings = {}
    
    # USA AI scripts start at 0x048800 (Bank $09:8800)
    # Each fighter has their AI data
    usa_ai_base = 0x048800
    
    # Search for AI script patterns in bank $09
    bank_09_start = 0x048000
    bank_09_end = 0x050000
    
    # Look for specific AI script markers
    # AI scripts often have specific byte patterns
    
    print(f"Searching Bank $09 (0x{bank_09_start:06X} - 0x{bank_09_end:06X})...")
    
    # Try to find by comparing to USA patterns
    for offset in range(0, 0x1000, 0x200):
        usa_pattern = usa_rom[usa_ai_base + offset:usa_ai_base + offset + 64]
        matches = find_pattern_all(rom_data, usa_pattern[:32], bank_09_start, bank_09_end)
        
        if matches:
            for addr in matches[:1]:
                print(f"  Potential AI script at PC 0x{addr:06X} ({pc_to_snes(addr)})")
    
    return findings

def compare_header_pointers(rom_type: str, rom_data: bytes, usa_rom: bytes) -> Dict:
    """Compare and analyze header pointer differences"""
    print(f"\n{'='*60}")
    print(f"HEADER POINTER ANALYSIS - {rom_type}")
    print(f"{'='*60}")
    
    # Fighter header table is at 0x048000
    # Each entry is 32 bytes
    # Offset 0x06-0x07: pose_table_ptr
    # Offset 0x08-0x09: ai_script_ptr
    # Offset 0x0A-0x0B: corner_man_ptr
    
    fighters = [
        "Gabby Jay", "Bear Hugger", "Piston Hurricane", "Bald Bull",
        "Bob Charlie", "Dragon Chan", "Masked Muscle", "Mr. Sandman",
        "Aran Ryan", "Heike Kagero", "Mad Clown", "Super Macho Man",
        "Narcis Prince", "Hoy Quarlow", "Rick Bruiser", "Nick Bruiser"
    ]
    
    print(f"\n{'Fighter':<20} {'Field':<15} {'USA Ptr':<12} {rom_type + ' Ptr':<12} {'Status'}")
    print("-" * 80)
    
    for i, fighter in enumerate(fighters):
        base = 0x048000 + (i * 32)
        
        # Read pointers
        usa_pose = struct.unpack('<H', usa_rom[base+6:base+8])[0]
        usa_ai = struct.unpack('<H', usa_rom[base+8:base+10])[0]
        usa_corner = struct.unpack('<H', usa_rom[base+10:base+12])[0]
        
        local_pose = struct.unpack('<H', rom_data[base+6:base+8])[0]
        local_ai = struct.unpack('<H', rom_data[base+8:base+10])[0]
        local_corner = struct.unpack('<H', rom_data[base+10:base+12])[0]
        
        pose_status = "SAME" if usa_pose == local_pose else "DIFF"
        ai_status = "SAME" if usa_ai == local_ai else "DIFF"
        corner_status = "SAME" if usa_corner == local_corner else "DIFF"
        
        print(f"{fighter:<20} {'pose_table':<15} ${usa_pose:04X}      ${local_pose:04X}      {pose_status}")
        print(f"{'':<20} {'ai_script':<15} ${usa_ai:04X}      ${local_ai:04X}      {ai_status}")
        print(f"{'':<20} {'corner_man':<15} ${usa_corner:04X}      ${local_corner:04X}      {corner_status}")
    
    return {}

def main():
    print("=" * 80)
    print("SUPER PUNCH-OUT!! EU/JPN TABLE ADDRESS FINDER")
    print("=" * 80)
    
    # Load ROMs
    with open(ROM_PATHS['USA'], 'rb') as f:
        usa_rom = f.read()
    with open(ROM_PATHS['EU'], 'rb') as f:
        eu_rom = f.read()
    with open(ROM_PATHS['JPN'], 'rb') as f:
        jpn_rom = f.read()
    
    all_findings = {
        'EU': {},
        'JPN': {}
    }
    
    # Analyze EU ROM
    print("\n" + "=" * 80)
    print("ANALYZING EUROPE (EU) ROM")
    print("=" * 80)
    
    all_findings['EU']['palettes'] = find_palette_tables('EU', eu_rom, usa_rom)
    all_findings['EU']['icons'] = find_icon_tables('EU', eu_rom, usa_rom)
    all_findings['EU']['circuits'] = find_circuit_tables('EU', eu_rom)
    all_findings['EU']['text'] = find_text_tables('EU', eu_rom)
    all_findings['EU']['ai_scripts'] = find_ai_script_tables('EU', eu_rom, usa_rom)
    compare_header_pointers('EU', eu_rom, usa_rom)
    
    # Analyze JPN ROM
    print("\n" + "=" * 80)
    print("ANALYZING JAPAN (JPN) ROM")
    print("=" * 80)
    
    all_findings['JPN']['palettes'] = find_palette_tables('JPN', jpn_rom, usa_rom)
    all_findings['JPN']['icons'] = find_icon_tables('JPN', jpn_rom, usa_rom)
    all_findings['JPN']['circuits'] = find_circuit_tables('JPN', jpn_rom)
    all_findings['JPN']['text'] = find_text_tables('JPN', jpn_rom)
    all_findings['JPN']['ai_scripts'] = find_ai_script_tables('JPN', jpn_rom, usa_rom)
    compare_header_pointers('JPN', jpn_rom, usa_rom)
    
    # Generate final summary
    print("\n" + "=" * 80)
    print("FINAL ADDRESS SUMMARY")
    print("=" * 80)
    
    print("\n{:<25} {:<20} {:<20} {:<15}".format("Table", "USA", "EU", "JPN"))
    print("-" * 85)
    
    # Fighter Header Table
    print("{:<25} {:<20} {:<20} {:<15}".format(
        "Fighter Header Table",
        "0x048000 ($89:8000)",
        "0x048000 ($89:8000)",
        "0x048000 ($89:8000)"
    ))
    
    # Palettes
    print("\nPALETTES:")
    for name in ["Gabby Jay", "Bear Hugger", "Piston Hurricane", "Bald Bull", "Bob Charlie"]:
        usa_addr = f"0x{0x06B9DA + (0x06BC3C - 0x06B9DA) * ['Gabby Jay', 'Bear Hugger', 'Piston Hurricane', 'Bald Bull', 'Bob Charlie'].index(name):06X}" if name in ["Gabby Jay", 'Bear Hugger', 'Piston Hurricane'] else "see docs"
        eu_info = all_findings['EU']['palettes'].get(name, {})
        jpn_info = all_findings['JPN']['palettes'].get(name, {})
        
        eu_addr = eu_info.get('snes', 'NOT FOUND') if eu_info else 'NOT FOUND'
        jpn_addr = jpn_info.get('snes', 'NOT FOUND') if jpn_info else 'NOT FOUND'
        
        print(f"  {name:<20} USA:{usa_addr:<15} EU:{eu_addr:<15} JPN:{jpn_addr}")
    
    # Save findings
    import json
    with open('eu_jpn_table_addresses.json', 'w') as f:
        json.dump(all_findings, f, indent=2)
    print("\nDetailed findings saved to: eu_jpn_table_addresses.json")

if __name__ == '__main__':
    main()
