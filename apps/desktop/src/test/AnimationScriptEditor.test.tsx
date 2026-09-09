import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { AnimationScriptEditor } from '../components/AnimationScriptEditor';

const store = vi.hoisted(() => ({ romSha1: 'usa', undoStack: [], refreshUndoState: vi.fn(), refreshPendingWrites: vi.fn() }));
vi.mock('../store/useStore', () => ({ useStore: () => store }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
afterEach(cleanup);
beforeEach(() => { vi.clearAllMocks(); store.romSha1 = 'usa'; store.undoStack = []; });

it('discards an earlier read when history changes during loading', async () => {
  type Instruction = { offset: number; opcode: number; operands: number[] };
  let finishOld!: (value: Instruction[]) => void;
  let reads = 0;
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === 'get_usa_animation_regions') return [{ id: 'gabby', label: 'Gabby' }];
    if (command === 'read_reference_animation') {
      if (++reads === 1) return new Promise(resolve => { finishOld = instructions => resolve({ instructions, references: [] }); });
      return { instructions: [{ offset: 0, opcode: 0x20, operands: [8, 20] }], references: [] };
    }
  });
  const view = render(<AnimationScriptEditor />);
  fireEvent.click(screen.getByText('Animation bytecode duration editor (experimental)'));
  await waitFor(() => expect(reads).toBe(1));
  store.undoStack = [];
  view.rerender(<AnimationScriptEditor />);
  await screen.findByRole('option', { name: /pose 8/ });
  await act(async () => { finishOld([{ offset: 0, opcode: 0x20, operands: [7, 12] }]); });
  expect(screen.queryByRole('option', { name: /pose 7/ })).not.toBeInTheDocument();
  fireEvent.change(screen.getByLabelText('Pose instruction'), { target: { value: '0' } });
  fireEvent.change(screen.getByLabelText('Duration byte (0-255)'), { target: { value: '30' } });
  store.undoStack = [];
  view.rerender(<AnimationScriptEditor />);
  await waitFor(() => expect(reads).toBe(3));
  expect(screen.getByLabelText('Duration byte (0-255)')).toHaveValue('');
  expect(screen.getByText('Apply duration')).toBeDisabled();
});

it('does not read or enable editing when source validation fails', async () => {
  vi.mocked(invoke).mockRejectedValue('Unsupported source ROM');
  render(<AnimationScriptEditor />);
  fireEvent.click(screen.getByText('Animation bytecode duration editor (experimental)'));
  expect(await screen.findByRole('alert')).toHaveTextContent('Unsupported source ROM');
  expect(invoke).not.toHaveBeenCalledWith('read_reference_animation', expect.anything());
  expect(screen.getByText('Apply duration')).toBeDisabled();
});

it('navigates verified references and labels unresolved destinations', async () => {
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === 'get_usa_animation_regions') return [{ id: 'a', label: 'A' }, { id: 'b', label: 'B' }];
    if (command === 'read_reference_animation') return { instructions: [{ offset: 0, opcode: 0x20, operands: [7, 12] }], references: [
      { source_region: 'a', target_region: 'b', reference: { instruction_offset: 3, target_address: 0x8100, resolution: 'instruction' } },
      { source_region: 'a', target_region: null, reference: { instruction_offset: 6, target_address: 0x8200, resolution: 'outside_interval' } },
    ] };
  });
  render(<AnimationScriptEditor />);
  fireEvent.click(screen.getByText('Animation bytecode duration editor (experimental)'));
  const link = await screen.findByRole('button', { name: 'Open B' });
  expect(screen.getByText(/outside interval/)).toBeInTheDocument();
  fireEvent.click(link);
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('read_reference_animation', { regionId: 'b' }));
  expect(screen.getByLabelText('Animation sequence')).toHaveValue('b');
});

it('submits a bounded duration with the complete stream preimage', async () => {
  vi.mocked(invoke).mockImplementation(async command => command === 'get_usa_animation_regions'
    ? [{ id: 'gabby_jay_098267', label: 'Gabby Jay sequence' }] : command === 'read_reference_animation'
    ? { instructions: [{ offset: 0, opcode: 0x22, operands: [] }, { offset: 1, opcode: 0x20, operands: [7, 12] }], references: [] } : undefined);
  render(<AnimationScriptEditor />);
  fireEvent.click(screen.getByText('Animation bytecode duration editor (experimental)'));
  await screen.findByRole('option', { name: /pose 7/ });
  fireEvent.change(screen.getByLabelText('Pose instruction'), { target: { value: '1' } });
  const input = screen.getByLabelText('Duration byte (0-255)');
  fireEvent.change(input, { target: { value: '256' } });
  expect(screen.getByText('Apply duration')).toBeDisabled();
  fireEvent.change(input, { target: { value: '24' } });
  fireEvent.click(screen.getByText('Apply duration'));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('update_reference_animation_duration', {
    expectedBytes: [0x22, 0x20, 7, 12], instructionOffset: 1, duration: 24,
    regionId: 'gabby_jay_098267',
  }));
  await waitFor(() => expect(store.refreshUndoState).toHaveBeenCalled());
  expect(store.refreshPendingWrites).toHaveBeenCalled();
});

it('stages multiple frames and submits one batch', async () => {
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === 'get_usa_animation_regions') return [{ id: 'gabby', label: 'Gabby' }];
    if (command === 'read_reference_animation') return { references: [], instructions: [
      { offset: 0, opcode: 0x20, operands: [7, 12] }, { offset: 3, opcode: 0x20, operands: [8, 16] },
    ] };
  });
  render(<AnimationScriptEditor />);
  fireEvent.click(screen.getByText('Animation bytecode duration editor (experimental)'));
  await screen.findByRole('option', { name: /pose 7/ });
  for (const [offset, duration] of [['0', '24'], ['3', '32']]) {
    fireEvent.change(screen.getByLabelText('Pose instruction'), { target: { value: offset } });
    fireEvent.change(screen.getByLabelText('Duration byte (0-255)'), { target: { value: duration } });
    fireEvent.click(screen.getByText('Stage duration'));
  }
  expect(screen.getByText('Apply duration')).toBeDisabled();
  expect(invoke).not.toHaveBeenCalledWith('update_animation_durations', expect.anything());
  fireEvent.click(screen.getByText('Apply staged durations'));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('update_animation_durations', {
    regionId: 'gabby', expectedBytes: [0x20, 7, 12, 0x20, 8, 16],
    edits: [{ instruction_offset: 0, duration: 24 }, { instruction_offset: 3, duration: 32 }],
  }));
  await waitFor(() => expect(screen.queryByRole('region', { name: 'Pending duration changes' })).not.toBeInTheDocument());
});

it('shows the partial timing trace and its explicit stop reason', async () => {
  vi.mocked(invoke).mockImplementation(async command => {
    if (command === 'get_usa_animation_regions') return [{ id: 'gabby', label: 'Gabby' }];
    if (command === 'read_reference_animation') return { instructions: [], references: [], timing: {
      events: [{ instruction_offset: 0, pose: 7, updates: 256,
        facing_request: 'left', position_request: { x_unmirrored: 130, x_mirrored: 126, y: 139 } }],
      stop: { reason: 'unknown_instruction', offset: 3 },
    } };
  });
  render(<AnimationScriptEditor />);
  fireEvent.click(screen.getByText('Animation bytecode duration editor (experimental)'));
  expect(await screen.findByRole('region', { name: 'Verified timing trace' })).toHaveTextContent('Stopped: unknown instruction at +3');
  expect(screen.getByText(/pose 7, 256 updates/)).toBeInTheDocument();
  expect(screen.getByText(/position request X 130 or 126, Y 139/)).toBeInTheDocument();
  expect(screen.getByText(/facing request left/)).toHaveTextContent('runtime mirroring not resolved');
});
