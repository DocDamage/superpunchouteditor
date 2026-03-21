/**
 * Tests for RegionSelector component – confirmation model behavior.
 *
 * Workstream 3 / Workstream 8 regression coverage:
 *   - Changing a radio option does NOT invoke onRegionSelected
 *   - "Open ROM" button invokes onRegionSelected exactly once per click
 *   - Force-load flow: warning → confirm → calls onRegionSelected exactly once
 *   - Cancelling the force-load dialog does NOT call onRegionSelected
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { RegionSelector } from '../components/RegionSelector';

// ─── Fixtures ────────────────────────────────────────────────────────────────

const MOCK_REGIONS = [
  {
    region: 'Usa' as const,
    display_name: 'Super Punch-Out!! (USA)',
    code: 'USA',
    is_supported: true,
    support_status: 'Fully supported',
    detected: false,
  },
  {
    region: 'Jpn' as const,
    display_name: 'Super Punch-Out!! (Japan)',
    code: 'JPN',
    is_supported: true,
    support_status: 'Fully supported',
    detected: false,
  },
  {
    region: 'Pal' as const,
    display_name: 'Super Punch-Out!! (PAL)',
    code: 'PAL',
    is_supported: true,
    support_status: 'Fully supported',
    detected: false,
  },
];

/** Simulates a successfully detected, fully-supported USA ROM. */
const SUPPORTED_DETECTION = {
  success: true,
  region: 'Usa' as const,
  display_name: 'Super Punch-Out!! (USA)',
  is_supported: true,
  sha1: 'aabbccddeeff00112233445566778899aabbccdd',
  error_message: null,
};

/** Simulates detection failure – unknown ROM, no region match. */
const UNSUPPORTED_DETECTION = {
  success: false,
  region: null,
  display_name: null,
  is_supported: false,
  sha1: 'ffffffffffffffffffffffffffffffffffffffff',
  error_message: 'Unknown ROM: SHA1 does not match any known Super Punch-Out!! version',
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

/** Set up invoke to resolve get_supported_regions then detect_rom_region. */
function mockInvokeSupportedRom() {
  vi.mocked(invoke)
    .mockResolvedValueOnce(MOCK_REGIONS)       // get_supported_regions (mount)
    .mockResolvedValueOnce(SUPPORTED_DETECTION); // detect_rom_region (romPath effect)
}

function mockInvokeUnsupportedRom() {
  vi.mocked(invoke)
    .mockResolvedValueOnce(MOCK_REGIONS)
    .mockResolvedValueOnce(UNSUPPORTED_DETECTION);
}

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('RegionSelector – confirmation-only model', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('changing a radio option does NOT call onRegionSelected', async () => {
    mockInvokeSupportedRom();
    const onRegionSelected = vi.fn();

    render(
      <RegionSelector
        romPath="/roms/usa.sfc"
        onRegionSelected={onRegionSelected}
      />
    );

    // Wait for region list to render.
    await waitFor(() =>
      screen.getByText('Super Punch-Out!! (Japan)')
    );

    const jpnRadio = screen.getByRole('radio', { name: /japan/i });
    fireEvent.click(jpnRadio);

    expect(onRegionSelected).not.toHaveBeenCalled();
  });

  it('"Open ROM" button calls onRegionSelected exactly once', async () => {
    mockInvokeSupportedRom();
    const onRegionSelected = vi.fn();

    render(
      <RegionSelector
        romPath="/roms/usa.sfc"
        onRegionSelected={onRegionSelected}
      />
    );

    // The button only appears once detection succeeds with a supported ROM.
    const openButton = await screen.findByRole('button', { name: /open rom/i });
    fireEvent.click(openButton);

    expect(onRegionSelected).toHaveBeenCalledTimes(1);
  });

  it('"Open ROM" is not rendered without a romPath', async () => {
    // Only get_supported_regions fires when there is no romPath.
    vi.mocked(invoke).mockResolvedValueOnce(MOCK_REGIONS);

    const onRegionSelected = vi.fn();
    render(<RegionSelector onRegionSelected={onRegionSelected} />);

    // Let the mount effect settle.
    await waitFor(() => screen.getByText('Super Punch-Out!! (USA)'));

    // No detection result means no "Open ROM" button.
    expect(screen.queryByRole('button', { name: /open rom/i })).toBeNull();
    expect(onRegionSelected).not.toHaveBeenCalled();
  });

  it('force-load "Confirm Force Load" calls onRegionSelected exactly once', async () => {
    mockInvokeUnsupportedRom();
    const onRegionSelected = vi.fn();

    render(
      <RegionSelector
        romPath="/roms/unknown.sfc"
        onRegionSelected={onRegionSelected}
        showForceLoad
      />
    );

    // The force-load section appears when detection fails.
    const forceBtn = await screen.findByRole('button', { name: /force load/i });
    fireEvent.click(forceBtn);

    // Confirmation dialog replaces the initial button.
    const confirmBtn = await screen.findByRole('button', { name: /confirm force load/i });
    fireEvent.click(confirmBtn);

    expect(onRegionSelected).toHaveBeenCalledTimes(1);
  });

  it('cancelling force-load does NOT call onRegionSelected', async () => {
    mockInvokeUnsupportedRom();
    const onRegionSelected = vi.fn();

    render(
      <RegionSelector
        romPath="/roms/unknown.sfc"
        onRegionSelected={onRegionSelected}
        showForceLoad
      />
    );

    const forceBtn = await screen.findByRole('button', { name: /force load/i });
    fireEvent.click(forceBtn);

    const cancelBtn = await screen.findByRole('button', { name: /cancel/i });
    fireEvent.click(cancelBtn);

    expect(onRegionSelected).not.toHaveBeenCalled();
  });

  it('force-load section does NOT appear for a supported ROM', async () => {
    mockInvokeSupportedRom();

    render(
      <RegionSelector
        romPath="/roms/usa.sfc"
        showForceLoad
      />
    );

    await screen.findByRole('button', { name: /open rom/i });
    expect(screen.queryByRole('button', { name: /force load/i })).toBeNull();
  });
});
