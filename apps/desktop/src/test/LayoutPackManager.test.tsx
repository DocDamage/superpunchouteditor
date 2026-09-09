import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { LayoutPackManager } from '../components/LayoutPackManager';

const store = vi.hoisted(() => ({
  boxers: [{ key: 'gabby_jay', name: 'Gabby Jay', unique_sprite_bins: [], shared_sprite_bins: [{}] }],
  selectedBoxer: { key: 'gabby_jay' }, refreshUndoState: vi.fn(), refreshPendingWrites: vi.fn(),
}));
vi.mock('../store/useStore', () => ({ useStore: Object.assign(() => store, { getState: () => store }) }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn(), save: vi.fn() }));
vi.mock('../components/ToastContainer', () => ({ showToast: vi.fn() }));
afterEach(cleanup);

describe('Layout pack installed paths', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation(async command => {
      if (command === 'get_available_layout_packs') return [{ filename: 'test.json', path: 'C:/AppData/layout-packs/test.json',
        name: 'Test pack', author: '', description: '', created_at: '', boxer_count: 1 }];
      if (command === 'import_layout_pack') return { layouts: [{ boxer_key: 'gabby_jay' }] };
      return undefined;
    });
  });

  it('uses the backend installed path and refreshes journal state after applying', async () => {
    render(<LayoutPackManager />);
    fireEvent.click(await screen.findByTitle('Apply this pack to boxers'));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith('apply_layout_pack', {
      packPath: 'C:/AppData/layout-packs/test.json', boxerKeys: ['gabby_jay'],
    }));
    expect(invoke).toHaveBeenCalledWith('import_layout_pack', { packPath: 'C:/AppData/layout-packs/test.json' });
    await waitFor(() => expect(store.refreshUndoState).toHaveBeenCalled());
    expect(store.refreshPendingWrites).toHaveBeenCalled();
  });

  it.each([false, true])('exports shared graphics only when selected: %s', async includeShared => {
    vi.mocked(save).mockResolvedValue('C:/exports/pack.json');
    render(<LayoutPackManager />);
    await screen.findByTitle('Apply this pack to boxers');
    fireEvent.click(screen.getByRole('button', { name: /Export Layouts/i }));
    fireEvent.change(screen.getByPlaceholderText('e.g., HD Sprite Layouts'), { target: { value: 'Selection test' } });
    const shared = screen.getByRole('checkbox', { name: /Include shared graphics/i });
    expect(shared).not.toBeChecked();
    if (includeShared) fireEvent.click(shared);
    fireEvent.click(screen.getByRole('button', { name: 'Export Pack' }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith('export_layout_pack', {
      boxerKeys: ['gabby_jay'], includeSharedFor: includeShared ? ['gabby_jay'] : [],
      metadata: { name: 'Selection test', author: '', description: '' }, outputPath: 'C:/exports/pack.json',
    }));
  });
});
