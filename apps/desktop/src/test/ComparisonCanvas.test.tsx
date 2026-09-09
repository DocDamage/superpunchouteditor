import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ComparisonCanvas } from '../components/ComparisonCanvas';

const store = vi.hoisted(() => ({
  comparison: { differences: [
    { type: 'Sprite', boxer: 'Gabby Jay', pc_offset: 32, bin_name: 'first', total_tiles: 1, changed_tile_indices: [0], tile_change_counts: { 0: 1 } },
    { type: 'Sprite', boxer: 'Gabby Jay', pc_offset: 64, bin_name: 'second', total_tiles: 1, changed_tile_indices: [0], tile_change_counts: { 0: 1 } },
  ] },
  renderComparisonView: vi.fn(),
}));
vi.mock('../store/useStore', () => ({ useStore: () => store }));
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('Comparison canvas render lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal('URL', { createObjectURL: vi.fn(() => 'blob:comparison'), revokeObjectURL: vi.fn() });
  });

  it('ignores obsolete render responses and revokes the current image on unmount', async () => {
    let first!: (bytes: Uint8Array) => void;
    store.renderComparisonView.mockImplementationOnce(() => new Promise(resolve => { first = resolve; }))
      .mockResolvedValueOnce(new Uint8Array([2]));
    const view = render(<ComparisonCanvas viewMode="overlay" selectedAsset="Sprite-0" />);
    view.rerender(<ComparisonCanvas viewMode="overlay" selectedAsset="Sprite-1" />);
    expect(await screen.findByAltText('Comparison')).toHaveAttribute('src', 'blob:comparison');
    await act(async () => { first(new Uint8Array([1])); });
    expect(URL.createObjectURL).toHaveBeenCalledTimes(1);
    view.unmount();
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:comparison');
  });

  it('revokes the previous image when selection is cleared', async () => {
    store.renderComparisonView.mockResolvedValue(new Uint8Array([1]));
    const view = render(<ComparisonCanvas viewMode="side-by-side" selectedAsset="Sprite-0" />);
    await screen.findByAltText('Comparison');
    view.rerender(<ComparisonCanvas viewMode="side-by-side" selectedAsset={null} />);
    await waitFor(() => expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:comparison'));
    expect(screen.queryByAltText('Comparison')).not.toBeInTheDocument();
  });

  it('drags the split image without requesting another backend render', async () => {
    store.renderComparisonView.mockResolvedValue(new Uint8Array([1]));
    render(<ComparisonCanvas viewMode="split" selectedAsset="Sprite-0" />);
    const image = await screen.findByAltText('Comparison');
    const layer = await screen.findByAltText('Original split layer');
    expect(store.renderComparisonView).toHaveBeenCalledTimes(2);
    vi.spyOn(image, 'getBoundingClientRect').mockReturnValue({ left: 0, width: 100 } as DOMRect);
    fireEvent.mouseDown(image);
    fireEvent.mouseMove(document, { clientX: 75 });
    fireEvent.mouseUp(document);
    expect(layer).toHaveStyle({ clipPath: 'inset(0 25% 0 0)' });
    expect(store.renderComparisonView).toHaveBeenCalledTimes(2);
  });
});
