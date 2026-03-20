/**
 * TypeScript types for Community Layout Packs
 * 
 * Layout packs allow users to import and export boxer layout JSON files,
 * enabling sharing curated layouts with the community.
 */

export interface LayoutPackMetadata {
  name: string;
  author: string;
  description: string;
}

export interface LayoutBin {
  filename: string;
  pc_offset: string;
  size: number;
  category: string;
  label?: string;
}

export interface PackBoxerLayout {
  boxer_key: string;
  version: string;
  layout_type: 'reference' | 'custom';
  bins: LayoutBin[];
  notes?: string;
}

export interface LayoutPack {
  version: string;
  name: string;
  author: string;
  description: string;
  created_at: string;
  layouts: PackBoxerLayout[];
}

export interface LayoutPackInfo {
  filename: string;
  name: string;
  author: string;
  description: string;
  created_at: string;
  boxer_count: number;
}

export interface BoxerValidation {
  boxer_key: string;
  exists_in_manifest: boolean;
  bins_valid: boolean;
  size_matches: boolean;
  warnings: string[];
  errors: string[];
}

export interface ValidationReport {
  valid: boolean;
  version_compatible: boolean;
  boxer_validations: BoxerValidation[];
  warnings: string[];
  errors: string[];
}

export interface BoxerLayoutComparison {
  boxer_key: string;
  current_bins: number;
  pack_bins: number;
  matching_bins: number;
  conflicts: string[];
}

export interface PackPreviewData {
  pack: LayoutPack;
  comparisons: BoxerLayoutComparison[];
  overall_compatible: boolean;
}

export interface ExportSelection {
  boxer_key: string;
  selected: boolean;
  include_shared: boolean;
}

export type LayoutPackSortField = 'name' | 'author' | 'created_at' | 'boxer_count';
export type LayoutPackSortOrder = 'asc' | 'desc';
