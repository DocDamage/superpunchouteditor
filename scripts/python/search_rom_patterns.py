#!/usr/bin/env python3
"""
Search for known data patterns in EU and JPN ROMs to identify table addresses.
Compares against USA ROM patterns from the manifest documentation.
"""

import os
import sys
import json
import struct
from pathlib import Path
from typing import List, Tuple, Dict, Optional
from dataclasses import dataclass

@dataclass
class PatternMatch:
    rom_type: str
    address: int
    pattern_type: str
    description: str
    confidence: str  # HIGH, MEDIUM, LOW
    data_preview: bytes
    usa_comparison: str

ROM_PATHS = {
    'USA': 'ROMs/Super Punch-Out!! (USA).sfc',
    'EU': 'ROMs/Super Punch-Out!! (Europe).sfc',
    'JPN': 'ROMs/Super Punch-Out!! (Japan) (NP).sfc'
}

# Known USA addresses for comparison
USA_ADDRESSES = {
    'fighter_header_table': 0x048000,  # Bank $09:8000
    'gabby_jay_palette': 0x06B9DA,     # $0D:B9DA
    'bear_hugger_palette': 0x06BC3C,   # $0D:BC3C
    'gabby_jay_icon': 0x06B7D8,        # $0D:B7D8
    'ai_script_table': 0x048800,       # Bank $09:8800
    'cornerman_text': 0x049000,        # $09:9000
}

# SNES LoROM conversion
def pc_to_snes(pc_addr: int) -> str:
    bank = (pc_addr // 0x8000) | 0x80
    offset = (pc_addr % 0x8000) | 0x8000
    return f"${bank:02X}:{offset:04X}"

def snes_to_pc(bank: int, offset: int) -> int:
    return ((bank & 0x7F) * 0x8000) + (offset & 0x7FFF)

class ROMPatternSearcher:
    def __init__(self, rom_path: str, rom_type: str):
        self.rom_path = rom_path
        self.rom_type = rom_type
        self.data = open(rom_path, 'rb').read()
        self.size = len(self.data)
        self.matches: List[PatternMatch] = []
        
    def read_bytes(self, addr: int, size: int) -> bytes:
        return self.data[addr:addr+size]
    
    def find_pattern(self, pattern: bytes, start: int = 0, end: Optional[int] = None) -> List[int]:
        """Find all occurrences of a byte pattern"""
        if end is None:
            end = self.size
        results = []
        idx = self.data.find(pattern, start, end)
        while idx != -1:
            results.append(idx)
            idx = self.data.find(pattern, idx + 1, end)
        return results
    
    def search_boxer_names(self):
        """Search for boxer names in ASCII/SNES text format"""
        # Common boxer names to search for
        boxer_names = [
            (b'GLASS', 'Glass Joe reference'),
            (b'BEAR', 'Bear Hugger reference'),
            (b'DRAGON', 'Dragon Chan reference'),
            (b'BALD', 'Bald Bull reference'),
            (b'PISTON', 'Piston Hurricane reference'),
            (b'MACHO', 'Super Macho Man reference'),
            (b'SANDMAN', 'Mr. Sandman reference'),
            (b'ARAN', 'Aran Ryan reference'),
            (b'NARCIS', 'Narcis Prince reference'),
            (b'HOY', 'Hoy Quarlow reference'),
            (b'BRUISER', 'Bruiser brothers reference'),
            (b'KAGERO', 'Heike Kagero reference'),
            (b'CLOWN', 'Mad Clown reference'),
            (b'CHARLIE', 'Bob Charlie reference'),
            (b'MUSCLE', 'Masked Muscle reference'),
        ]
        
        print(f"\n=== Searching for Boxer Names in {self.rom_type} ===")
        for name, desc in boxer_names:
            # Search for uppercase (common in SNES games)
            results = self.find_pattern(name)
            if results:
                for addr in results[:3]:  # Limit to first 3 matches
                    preview = self.read_bytes(addr, min(32, self.size - addr))
                    match = PatternMatch(
                        rom_type=self.rom_type,
                        address=addr,
                        pattern_type='boxer_name',
                        description=desc,
                        confidence='MEDIUM',
                        data_preview=preview,
                        usa_comparison=f"Searching for '{name.decode()}'"
                    )
                    self.matches.append(match)
                    print(f"  Found '{name.decode()}' at PC 0x{addr:06X} ({pc_to_snes(addr)})")
                    print(f"    Preview: {preview[:16].hex()}")
    
    def search_palette_signatures(self):
        """Search for palette data signatures (BGR555 format)"""
        # SNES BGR555 palettes have specific characteristics
        # Common palette patterns - look for sequences that could be palette data
        print(f"\n=== Searching for Palette Signatures in {self.rom_type} ===")
        
        # Search in bank $0D region (where palettes are in USA)
        bank_0d_start = snes_to_pc(0x0D, 0x8000)  # 0x068000
        bank_0d_end = snes_to_pc(0x0D, 0xFFFF)    # 0x06FFFF
        
        # Look for the Gabby Jay palette pattern from USA (96 bytes at 0x06B9DA)
        if self.rom_type == 'USA':
            usa_palette = self.read_bytes(0x06B9DA, 96)
            print(f"  USA Gabby Jay palette (96 bytes from 0x06B9DA): {usa_palette[:16].hex()}...")
        
        # Search for common palette patterns (repeated color values)
        # Palettes often have 0x00 0x00 (transparent/black) at start
        palette_starts = self.find_pattern(b'\x00\x00', bank_0d_start, bank_0d_end)
        
        print(f"  Found {len(palette_starts)} potential palette starts in bank $0D")
        
        # Look for specific palette sizes (96 bytes = 48 colors, 128 bytes = 64 colors)
        for addr in palette_starts[:10]:
            preview = self.read_bytes(addr, 32)
            # Check if looks like valid BGR555 data
            valid_colors = 0
            for i in range(0, min(32, len(preview) - 1), 2):
                color = preview[i] | (preview[i+1] << 8)
                # BGR555 colors have bit 15 as 0 (no priority bit in palette data)
                if color < 0x8000:
                    valid_colors += 1
            
            if valid_colors >= 8:  # At least 8 valid-looking colors
                match = PatternMatch(
                    rom_type=self.rom_type,
                    address=addr,
                    pattern_type='palette_data',
                    description=f'Potential palette data ({valid_colors} valid colors in preview)',
                    confidence='MEDIUM',
                    data_preview=preview,
                    usa_comparison=f"USA Gabby Jay palette at 0x06B9DA"
                )
                self.matches.append(match)
                if len([m for m in self.matches if m.pattern_type == 'palette_data']) <= 5:
                    print(f"  Potential palette at PC 0x{addr:06X} ({pc_to_snes(addr)})")
    
    def search_fighter_header_table(self):
        """Search for fighter header table patterns"""
        print(f"\n=== Searching for Fighter Header Table in {self.rom_type} ===")
        
        # In USA, fighter header table is at 0x048000 (Bank $09:8000)
        # Each entry is 32 bytes, 16 fighters = 512 bytes total
        
        # Look for bank $09 pattern - fighters usually have specific stats
        bank_09_start = snes_to_pc(0x09, 0x8000)  # 0x048000
        
        if self.rom_type == 'USA':
            # Read the USA header table for pattern matching
            usa_header = self.read_bytes(0x048000, 512)
            print(f"  USA header table first 32 bytes (Gabby Jay): {usa_header[:32].hex()}")
            
            # Look for similar patterns in other ROMs
            for rom_type in ['EU', 'JPN']:
                if rom_type != self.rom_type:
                    continue
        
        # Search strategy: Look for patterns that look like fighter headers
        # Fighter headers have specific structure:
        # - Byte 0: Palette ID (usually small number 0-15)
        # - Byte 1: Attack power (varies)
        # - Byte 2: Defense rating (varies)
        # - Byte 3: Speed rating (varies)
        
        # Search for byte 0 being 0x00-0x0F and following bytes having reasonable values
        candidates = []
        for i in range(bank_09_start, min(bank_09_start + 0x10000, self.size - 32), 32):
            data = self.read_bytes(i, 32)
            palette_id = data[0]
            attack = data[1]
            defense = data[2]
            speed = data[3]
            
            # Reasonable fighter stats checks
            if palette_id <= 0x20 and attack <= 0xFF and defense <= 0xFF and speed <= 0xFF:
                # Check for pointer-like data at expected offsets (0x06-0x0B)
                ptr1 = data[6] | (data[7] << 8)
                ptr2 = data[8] | (data[9] << 8)
                ptr3 = data[10] | (data[11] << 8)
                
                # Pointers should be in valid SNES address ranges
                if 0x8000 <= ptr1 <= 0xFFFF and 0x8000 <= ptr2 <= 0xFFFF:
                    candidates.append((i, data))
        
        print(f"  Found {len(candidates)} potential fighter header candidates in bank $09")
        
        if len(candidates) >= 16:  # We expect 16 fighters
            # Check if they form a consistent table
            first_addr = candidates[0][0]
            last_addr = candidates[-1][0]
            expected_last = first_addr + (15 * 32)  # 16 fighters, 32 bytes each
            
            if abs(last_addr - expected_last) < 32:
                match = PatternMatch(
                    rom_type=self.rom_type,
                    address=first_addr,
                    pattern_type='fighter_header_table',
                    description=f'Potential fighter header table ({len(candidates)} entries)',
                    confidence='HIGH' if len(candidates) == 16 else 'MEDIUM',
                    data_preview=candidates[0][1],
                    usa_comparison="USA at 0x048000"
                )
                self.matches.append(match)
                print(f"  HIGH CONFIDENCE: Fighter header table at PC 0x{first_addr:06X} ({pc_to_snes(first_addr)})")
    
    def search_text_tables(self):
        """Search for text/string table markers"""
        print(f"\n=== Searching for Text Table Markers in {self.rom_type} ===")
        
        # Look for common SNES text patterns
        # Text often uses specific encoding or has length prefixes
        
        # Search for common strings
        text_patterns = [
            (b'CONTINUE', 'Continue text'),
            (b'RETRY', 'Retry text'),
            (b'WIN', 'Win text'),
            (b'LOSE', 'Lose text'),
            (b'KO', 'KO text'),
            (b'TKO', 'TKO text'),
            (b'ROUND', 'Round text'),
        ]
        
        for pattern, desc in text_patterns:
            results = self.find_pattern(pattern)
            if results:
                for addr in results[:2]:
                    preview = self.read_bytes(addr, 32)
                    match = PatternMatch(
                        rom_type=self.rom_type,
                        address=addr,
                        pattern_type='text_marker',
                        description=desc,
                        confidence='LOW',
                        data_preview=preview,
                        usa_comparison='N/A'
                    )
                    self.matches.append(match)
                    print(f"  Found '{pattern.decode()}' at PC 0x{addr:06X}")
    
    def search_pointer_tables(self):
        """Search for pointer tables (16-bit values that look like SNES addresses)"""
        print(f"\n=== Searching for Pointer Tables in {self.rom_type} ===")
        
        # Pointer tables often contain multiple 16-bit values in sequence
        # that fall within valid SNES address ranges ($8000-$FFFF)
        
        bank_09_start = snes_to_pc(0x09, 0x8000)
        bank_09_end = snes_to_pc(0x09, 0xFFFF)
        
        # Scan for sequences of valid pointers
        potential_tables = []
        i = bank_09_start
        while i < bank_09_end - 32:
            valid_ptrs = 0
            for j in range(0, 32, 2):
                ptr = self.data[i+j] | (self.data[i+j+1] << 8)
                if 0x8000 <= ptr <= 0xFFFF:
                    valid_ptrs += 1
            
            if valid_ptrs >= 8:  # At least 8 valid pointers in 32 bytes
                potential_tables.append((i, valid_ptrs))
                i += 32  # Skip ahead
            else:
                i += 2
        
        print(f"  Found {len(potential_tables)} potential pointer tables in bank $09")
        
        # Group consecutive tables
        if potential_tables:
            for addr, count in potential_tables[:5]:
                preview = self.read_bytes(addr, 32)
                match = PatternMatch(
                    rom_type=self.rom_type,
                    address=addr,
                    pattern_type='pointer_table',
                    description=f'Potential pointer table ({count} valid pointers)',
                    confidence='MEDIUM',
                    data_preview=preview,
                    usa_comparison='N/A'
                )
                self.matches.append(match)
                print(f"  Pointer table at PC 0x{addr:06X} ({pc_to_snes(addr)}): {count} valid pointers")
    
    def search_ai_script_tables(self):
        """Search for AI script table patterns"""
        print(f"\n=== Searching for AI Script Tables in {self.rom_type} ===")
        
        # In USA, AI scripts are at 0x048800 onwards (Bank $09:8800)
        # AI scripts often have specific byte patterns
        
        bank_09_start = snes_to_pc(0x09, 0x8000)
        
        # Look for common AI script patterns
        # AI scripts often start with specific opcodes or have NOP-like patterns
        ai_signatures = [
            b'\x00\x00\x00\x00',  # NOP-like sequences
            b'\xFF\xFF\xFF\xFF',  # End markers
        ]
        
        for sig in ai_signatures:
            results = self.find_pattern(sig, bank_09_start, bank_09_start + 0x8000)
            if results:
                for addr in results[:3]:
                    preview = self.read_bytes(addr, 32)
                    match = PatternMatch(
                        rom_type=self.rom_type,
                        address=addr,
                        pattern_type='ai_script_marker',
                        description=f'Potential AI script marker',
                        confidence='LOW',
                        data_preview=preview,
                        usa_comparison='USA AI scripts at 0x048800'
                    )
                    self.matches.append(match)
    
    def compare_with_usa(self):
        """Compare ROM structure with USA version"""
        print(f"\n=== Comparing {self.rom_type} with USA ===")
        
        if self.rom_type == 'USA':
            return
        
        # Load USA data
        usa_data = open(ROM_PATHS['USA'], 'rb').read()
        
        # Compare specific known regions
        comparisons = [
            (0x048000, 512, 'Fighter Header Table'),
            (0x06B9DA, 96, 'Gabby Jay Palette'),
            (0x06BC3C, 96, 'Bear Hugger Palette'),
        ]
        
        for addr, size, desc in comparisons:
            if addr + size > self.size or addr + size > len(usa_data):
                continue
            
            usa_bytes = usa_data[addr:addr+size]
            local_bytes = self.data[addr:addr+size]
            
            if usa_bytes == local_bytes:
                print(f"  {desc}: IDENTICAL at 0x{addr:06X}")
                match = PatternMatch(
                    rom_type=self.rom_type,
                    address=addr,
                    pattern_type='identical_to_usa',
                    description=f'{desc} matches USA exactly',
                    confidence='HIGH',
                    data_preview=local_bytes[:32],
                    usa_comparison=f'Same as USA at 0x{addr:06X}'
                )
                self.matches.append(match)
            else:
                # Calculate similarity
                matches = sum(1 for a, b in zip(usa_bytes, local_bytes) if a == b)
                similarity = matches / size * 100
                print(f"  {desc}: {similarity:.1f}% similar at 0x{addr:06X}")
                
                if similarity > 80:
                    match = PatternMatch(
                        rom_type=self.rom_type,
                        address=addr,
                        pattern_type='similar_to_usa',
                        description=f'{desc} similar to USA ({similarity:.1f}%)',
                        confidence='MEDIUM',
                        data_preview=local_bytes[:32],
                        usa_comparison=f'USA at 0x{addr:06X}'
                    )
                    self.matches.append(match)
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive report of findings"""
        report = {
            'rom_type': self.rom_type,
            'rom_size': self.size,
            'matches': []
        }
        
        for match in self.matches:
            report['matches'].append({
                'rom_type': match.rom_type,
                'address': f'0x{match.address:06X}',
                'snes_address': pc_to_snes(match.address),
                'pattern_type': match.pattern_type,
                'description': match.description,
                'confidence': match.confidence,
                'data_preview': match.data_preview[:16].hex(),
                'usa_comparison': match.usa_comparison
            })
        
        return report


def main():
    print("=" * 80)
    print("Super Punch-Out!! ROM Pattern Search")
    print("Comparing EU and JPN ROMs against USA patterns")
    print("=" * 80)
    
    all_reports = []
    
    for rom_type, rom_path in ROM_PATHS.items():
        if not os.path.exists(rom_path):
            print(f"WARNING: {rom_path} not found, skipping...")
            continue
        
        print(f"\n{'='*80}")
        print(f"Processing {rom_type} ROM: {rom_path}")
        print(f"{'='*80}")
        
        searcher = ROMPatternSearcher(rom_path, rom_type)
        
        # Run all searches
        searcher.search_boxer_names()
        searcher.search_palette_signatures()
        searcher.search_fighter_header_table()
        searcher.search_text_tables()
        searcher.search_pointer_tables()
        searcher.search_ai_script_tables()
        searcher.compare_with_usa()
        
        report = searcher.generate_report()
        all_reports.append(report)
    
    # Generate summary
    print("\n" + "=" * 80)
    print("SUMMARY REPORT")
    print("=" * 80)
    
    for report in all_reports:
        print(f"\n{report['rom_type']} ROM:")
        print(f"  Size: {report['rom_size']:,} bytes")
        print(f"  Total matches: {len(report['matches'])}")
        
        # Group by confidence
        high_conf = [m for m in report['matches'] if m['confidence'] == 'HIGH']
        med_conf = [m for m in report['matches'] if m['confidence'] == 'MEDIUM']
        low_conf = [m for m in report['matches'] if m['confidence'] == 'LOW']
        
        print(f"  HIGH confidence: {len(high_conf)}")
        print(f"  MEDIUM confidence: {len(med_conf)}")
        print(f"  LOW confidence: {len(low_conf)}")
    
    # Save detailed report to JSON
    output_path = 'rom_pattern_search_results.json'
    with open(output_path, 'w') as f:
        json.dump(all_reports, f, indent=2)
    print(f"\nDetailed results saved to: {output_path}")
    
    # Generate comparison table
    print("\n" + "=" * 80)
    print("ADDRESS COMPARISON TABLE")
    print("=" * 80)
    
    print("\n{:<20} {:<15} {:<15} {:<15} {:<20}".format(
        "Pattern Type", "USA", "EU", "JPN", "Status"
    ))
    print("-" * 90)
    
    # Key addresses to compare
    key_patterns = [
        'fighter_header_table',
        'palette_data',
        'boxer_name',
        'identical_to_usa'
    ]
    
    for pattern in key_patterns:
        for report in all_reports:
            matches = [m for m in report['matches'] if m['pattern_type'] == pattern]
            for match in matches[:3]:  # Limit output
                print("{:<20} {:<15} {:<15} {:<15} {:<20}".format(
                    match['pattern_type'][:19],
                    match.get('usa_comparison', 'N/A')[-14:] if len(match.get('usa_comparison', 'N/A')) > 14 else match.get('usa_comparison', 'N/A'),
                    match['address'] if report['rom_type'] == 'EU' else '-',
                    match['address'] if report['rom_type'] == 'JPN' else '-',
                    match['confidence']
                ))

if __name__ == '__main__':
    main()
