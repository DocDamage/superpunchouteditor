import { render, screen, fireEvent, waitFor, cleanup } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { AiScriptEditor } from '../components/AiScriptEditor';

const store = vi.hoisted(() => ({
  romSha1: 'usa', undoStack: [], refreshUndoState: vi.fn(), refreshPendingWrites: vi.fn(),
}));
vi.mock('../store/useStore', () => ({ useStore: () => store }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
afterEach(cleanup);

describe('AI instruction editor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation(async command => {
      if (command === 'get_usa_ai_script_regions') return [{ id: 'gabby', fighter: 'Gabby Jay', pc_offset: 0x480ae, length: 3 }];
      return [{ offset: 0, opcode: 42, operands: [30] }, { offset: 2, opcode: 0, operands: [] }];
    });
  });

  it('submits exact expected bytes and rejects malformed operands', async () => {
    render(<AiScriptEditor />);
    fireEvent.click(screen.getByText('AI instruction editor (experimental)'));
    fireEvent.click(await screen.findByText(/Wait frames/));
    const input = screen.getByLabelText('Wait frames');
    fireEvent.change(input, { target: { value: '256' } });
    expect(screen.getByText('Apply operands')).toBeDisabled();
    fireEvent.change(input, { target: { value: '60' } });
    fireEvent.click(screen.getByText('Apply operands'));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith('update_ai_script_operands', {
      pcOffset: 0x480ae, expectedBytes: [42, 30, 0], instructionOffset: 0, operands: [60],
    }));
    await waitFor(() => expect(store.refreshUndoState).toHaveBeenCalled());
    expect(store.refreshPendingWrites).toHaveBeenCalled();
  });

  it('shows backend compatibility rejection without enabling editing', async () => {
    vi.mocked(invoke).mockRejectedValue('Unsupported ROM source');
    render(<AiScriptEditor />);
    fireEvent.click(screen.getByText('AI instruction editor (experimental)'));
    expect(await screen.findByRole('alert')).toHaveTextContent('Unsupported ROM source');
    expect(screen.getByLabelText('Fighter script')).toBeDisabled();
    expect(screen.queryByText('Apply operands')).not.toBeInTheDocument();
  });

  it('offers only local instruction boundaries and preserves an external call target', async () => {
    vi.mocked(invoke).mockImplementation(async command => {
      if (command === 'get_usa_ai_script_regions') return [{ id: 'gabby', fighter: 'Gabby Jay', pc_offset: 0x480ae, length: 6 }];
      return [{ offset: 0, opcode: 42, operands: [30] },
        { offset: 2, opcode: 68, operands: [0, 0x90] }, { offset: 5, opcode: 0, operands: [] }];
    });
    render(<AiScriptEditor />);
    fireEvent.click(screen.getByText('AI instruction editor (experimental)'));
    fireEvent.click(await screen.findByText(/Call subroutine/));
    const destination = screen.getByLabelText('Call destination') as HTMLSelectElement;
    expect(destination.value).toBe(String(0x9000));
    expect(Array.from(destination.options).map(option => Number(option.value)))
      .toEqual([0x9000, 0x80ae, 0x80b0, 0x80b3]);
    expect(screen.getByText('Apply operands')).toBeDisabled();
    fireEvent.change(destination, { target: { value: String(0x80b3) } });
    fireEvent.click(screen.getByText('Apply operands'));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith('update_ai_script_operands', {
      pcOffset: 0x480ae, expectedBytes: [42, 30, 68, 0, 0x90, 0], instructionOffset: 2, operands: [0xb3, 0x80],
    }));
  });

  it('reloads on journal history changes and drops the previous draft', async () => {
    let frames = 30;
    vi.mocked(invoke).mockImplementation(async command => {
      if (command === 'get_usa_ai_script_regions') return [{ id: 'gabby', fighter: 'Gabby Jay', pc_offset: 0x480ae, length: 3 }];
      return [{ offset: 0, opcode: 42, operands: [frames] }, { offset: 2, opcode: 0, operands: [] }];
    });
    const view = render(<AiScriptEditor />);
    fireEvent.click(screen.getByText('AI instruction editor (experimental)'));
    fireEvent.click(await screen.findByText(/Wait frames/));
    fireEvent.change(screen.getByLabelText('Wait frames'), { target: { value: '60' } });
    frames = 45;
    store.undoStack = [];
    view.rerender(<AiScriptEditor />);
    await screen.findByText(/Wait frames \[2d\]/);
    expect(screen.queryByLabelText('Wait frames')).not.toBeInTheDocument();
    fireEvent.click(screen.getByText(/Wait frames \[2d\]/));
    expect(screen.getByLabelText('Wait frames')).toHaveValue(45);
    expect(screen.getByText('Apply operands')).toBeDisabled();
  });
});
