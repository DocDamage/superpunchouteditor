#!/usr/bin/env python3
"""Detailed SNES header analysis"""

import sys

def analyze_header(data, base, name):
    print(f'=== {name} SNES Header at 0x{base:04X} ===')
    
    maker_code = data[base:base+2]
    print(f'  Maker code: {maker_code} ({maker_code.hex()})')
    
    game_code = data[base+2:base+6]
    print(f'  Game code: {game_code} ({game_code.hex()})')
    
    title_bytes = data[base+0x10:base+0x25]
    title = ''.join(chr(b) if 32 <= b < 127 else '?' for b in title_bytes).strip()
    print(f'  Title: "{title}"')
    
    map_mode = data[base+0x25]
    print(f'  Map mode: 0x{map_mode:02X} ({"HiROM" if map_mode & 0x01 else "LoROM"})')
    
    rom_type = data[base+0x26]
    print(f'  ROM type: 0x{rom_type:02X}')
    
    rom_size_exp = data[base+0x27]
    rom_size = 1024 << rom_size_exp
    print(f'  ROM size: 2^{rom_size_exp} = {rom_size:,} bytes ({rom_size//1024}KB)')
    
    sram_exp = data[base+0x28]
    sram_size = 1024 << sram_exp if sram_exp else 0
    print(f'  SRAM size: 2^{sram_exp} = {sram_size:,} bytes')
    
    regions = {0: 'Japan', 1: 'North America', 2: 'Europe'}
    region = data[base+0x29]
    print(f'  Region: 0x{region:02X} ({regions.get(region, "Other")})')
    
    print(f'  Version: 1.{data[base+0x2A]}')
    
    checksum_comp = (data[base+0x2C] << 8) | data[base+0x2B]
    checksum = (data[base+0x2E] << 8) | data[base+0x2D]
    print(f'  Checksum comp: 0x{checksum_comp:04X}')
    print(f'  Checksum: 0x{checksum:04X}')
    print(f'  Checksum valid: {"YES" if (checksum ^ checksum_comp) == 0xFFFF else "NO"}')
    print()

def main():
    for name in ['USA', 'Europe', 'Japan']:
        fname = f'Super Punch-Out!! ({name}).sfc' if name != 'Japan' else 'Super Punch-Out!! (Japan) (NP).sfc'
        with open(fname, 'rb') as f:
            data = f.read()
        
        print(f'\n{"="*60}')
        print(f'ROM: {name}')
        print(f'{"="*60}')
        
        # Try both header locations
        analyze_header(data, 0x7FB0, f'{name} (LoROM location)')
        analyze_header(data, 0xFFB0, f'{name} (HiROM location)')

if __name__ == '__main__':
    main()
