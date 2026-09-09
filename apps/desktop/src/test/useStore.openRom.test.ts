/**
 * Tests for useStore openRom() state reset behavior.
 *
 * These are the regression tests called out in the technical debt plan
 * (Workstream 1 / Workstream 8):
 *
 *   - Opening a ROM clears stale boxer selection
 *   - Opening a ROM clears palette state
 *   - Opening a ROM reloads the boxer list
 *   - Sequential ROM loads remain stable and do not leave stale data
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';

// Minimal faker for a BoxerRecord.
const makeBoxer = (key: string) => ({
  name: key,
  key,
  reference_sheet: '',
  palette_files: [],
  icon_files: [],
  portrait_files: [],
  large_portrait_files: [],
  unique_sprite_bins: [],
  shared_sprite_bins: [],
  other_files: [],
});

const SHA1_USA = 'aaaa1111';
const SHA1_JPN = 'bbbb2222';

const USA_BOXERS = [makeBoxer('Little Mac'), makeBoxer('Bear Hugger')];
const JPN_BOXERS = [makeBoxer('Mac'), makeBoxer('Gabby Jay')];

describe('useStore – openRom() state transitions', () => {
  beforeEach(() => {
    // Reset the Zustand store to its initial state between tests.
    useStore.setState({
      romSha1: null,
      boxers: [],
      selectedBoxer: null,
      currentPalette: null,
      currentPaletteOffset: null,
      fighters: [],
      selectedFighterId: null,
      poses: [],
      frames: [],
      currentFrame: null,
      currentFrameIndex: 0,
      pendingWrites: new Set(),
      error: null,
      canUndo: false,
      canRedo: false,
      undoStack: [],
      redoStack: [],
      editHistory: [],
    });

    vi.clearAllMocks();
  });

  it('loads boxers from the new manifest after a successful ROM open', async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(SHA1_USA) // open_rom
      .mockResolvedValueOnce(USA_BOXERS) // get_boxers
      .mockResolvedValueOnce([]) // can_undo  \
      .mockResolvedValueOnce([]) // can_redo   | refreshUndoState
      .mockResolvedValueOnce([]) // get_undo_stack
      .mockResolvedValueOnce([]); // get_redo_stack

    await useStore.getState().openRom('/path/to/usa.sfc');

    expect(useStore.getState().romSha1).toBe(SHA1_USA);
    expect(useStore.getState().boxers).toEqual(USA_BOXERS);
  });

  it('clears selectedBoxer when a new ROM is opened', async () => {
    // Pre-load a boxer from a previous ROM.
    useStore.setState({ selectedBoxer: makeBoxer('OldBoxer'), romSha1: 'old' });

    vi.mocked(invoke)
      .mockResolvedValueOnce(SHA1_USA)
      .mockResolvedValueOnce(USA_BOXERS)
      .mockResolvedValue([]); // undo state calls

    await useStore.getState().openRom('/path/to/usa.sfc');

    expect(useStore.getState().selectedBoxer).toBeNull();
  });

  it('clears palette state when a new ROM is opened', async () => {
    useStore.setState({
      currentPalette: [{ r: 255, g: 0, b: 0 }] as any,
      currentPaletteOffset: '0x1234',
    });

    vi.mocked(invoke)
      .mockResolvedValueOnce(SHA1_USA)
      .mockResolvedValueOnce(USA_BOXERS)
      .mockResolvedValue([]);

    await useStore.getState().openRom('/path/to/usa.sfc');

    expect(useStore.getState().currentPalette).toBeNull();
    expect(useStore.getState().currentPaletteOffset).toBeNull();
  });

  it('clears pendingWrites when a new ROM is opened', async () => {
    useStore.setState({ pendingWrites: new Set(['0x1000', '0x2000']) });

    vi.mocked(invoke)
      .mockResolvedValueOnce(SHA1_USA)
      .mockResolvedValueOnce(USA_BOXERS)
      .mockResolvedValue([]);

    await useStore.getState().openRom('/path/to/usa.sfc');

    expect(useStore.getState().pendingWrites.size).toBe(0);
  });

  it('resets frame state when a new ROM is opened', async () => {
    useStore.setState({
      frames: [{ id: 1 }] as any,
      currentFrame: { id: 1 } as any,
      currentFrameIndex: 3,
    });

    vi.mocked(invoke)
      .mockResolvedValueOnce(SHA1_USA)
      .mockResolvedValueOnce(USA_BOXERS)
      .mockResolvedValue([]);

    await useStore.getState().openRom('/path/to/usa.sfc');

    expect(useStore.getState().frames).toEqual([]);
    expect(useStore.getState().currentFrame).toBeNull();
    expect(useStore.getState().currentFrameIndex).toBe(0);
  });

  it('sequential ROM loads (USA → JPN) never leave stale boxer data', async () => {
    // First load: USA ROM
    vi.mocked(invoke)
      .mockResolvedValueOnce(SHA1_USA)
      .mockResolvedValueOnce(USA_BOXERS)
      .mockResolvedValue([]);

    await useStore.getState().openRom('/roms/usa.sfc');
    // Simulate boxer selection during USA session.
    useStore.setState({ selectedBoxer: USA_BOXERS[0] });
    expect(useStore.getState().boxers).toEqual(USA_BOXERS);

    // Second load: JPN ROM — must replace boxer list and clear selection.
    vi.mocked(invoke)
      .mockResolvedValueOnce(SHA1_JPN)
      .mockResolvedValueOnce(JPN_BOXERS)
      .mockResolvedValue([]);

    await useStore.getState().openRom('/roms/jpn.sfc');

    expect(useStore.getState().romSha1).toBe(SHA1_JPN);
    expect(useStore.getState().boxers).toEqual(JPN_BOXERS);
    expect(useStore.getState().selectedBoxer).toBeNull();
    // No USA boxers should remain.
    const boxerKeys = useStore.getState().boxers.map((b) => b.key);
    expect(boxerKeys).not.toContain('Little Mac');
    expect(boxerKeys).not.toContain('Bear Hugger');
  });

  it('sets an error and leaves romSha1 null when open_rom fails', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Unknown ROM region'));

    await useStore.getState().openRom('/path/to/unknown.sfc');

    expect(useStore.getState().romSha1).toBeNull();
    expect(useStore.getState().error).toMatch(/Unknown ROM region/i);
    // Boxer list must be empty after a failed load — no stale data.
    expect(useStore.getState().boxers).toEqual([]);
  });

  it('reloading the same ROM is stable (idempotent boxer list)', async () => {
    for (let i = 0; i < 2; i++) {
      vi.mocked(invoke)
        .mockResolvedValueOnce(SHA1_USA)
        .mockResolvedValueOnce(USA_BOXERS)
        .mockResolvedValue([]);

      await useStore.getState().openRom('/roms/usa.sfc');
    }

    expect(useStore.getState().boxers).toEqual(USA_BOXERS);
    expect(useStore.getState().romSha1).toBe(SHA1_USA);
  });
});
