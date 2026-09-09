import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';

interface Region { id: string; fighter: string; pc_offset: number; length: number }
interface Instruction { offset: number; opcode: number; operands: number[] }
const hex = (bytes: number[]) => bytes.map(byte => byte.toString(16).padStart(2, '0')).join(' ');
const operation = (opcode: number) => ({ 0: 'Return', 42: 'Wait frames', 68: 'Call subroutine' }[opcode]
  ?? `Unknown operation $${hex([opcode])}`);

export function AiScriptEditor() {
  const { romSha1, undoStack, refreshUndoState, refreshPendingWrites } = useStore();
  const [regions, setRegions] = useState<Region[]>([]);
  const [regionId, setRegionId] = useState('');
  const [instructions, setInstructions] = useState<Instruction[]>([]);
  const [selectedOffset, setSelectedOffset] = useState<number | null>(null);
  const [draft, setDraft] = useState('');
  const [waitFrames, setWaitFrames] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [reload, setReload] = useState(0);
  const request = useRef(0);
  const region = regions.find(entry => entry.id === regionId);
  const selected = instructions.find(entry => entry.offset === selectedOffset);

  useEffect(() => {
    let cancelled = false;
    setRegions([]);
    setRegionId('');
    setError('');
    invoke<Region[]>('get_usa_ai_script_regions').then(result => {
      if (cancelled) return;
      setRegions(result);
      setRegionId(result[0]?.id ?? '');
    }).catch(reason => { if (!cancelled) setError(String(reason)); });
    return () => { cancelled = true; };
  }, [romSha1]);

  useEffect(() => {
    const token = ++request.current;
    setInstructions([]);
    setSelectedOffset(null);
    setDraft('');
    setBusy(false);
    if (!region) return;
    setBusy(true);
    setError('');
    invoke<Instruction[]>('read_ai_script_stream', { pcOffset: region.pc_offset, length: region.length })
      .then(result => { if (request.current === token) setInstructions(result); })
      .catch(reason => { if (request.current === token) setError(String(reason)); })
      .finally(() => { if (request.current === token) setBusy(false); });
    return () => { request.current++; };
  }, [region, romSha1, undoStack, reload]);

  const waitValid = /^\d+$/.test(waitFrames) && Number(waitFrames) <= 255;
  const words = selected?.opcode === 0x2a
    ? (waitValid ? [Number(waitFrames).toString(16).padStart(2, '0')] : [])
    : (draft.trim() ? draft.trim().split(/\s+/) : []);
  const valid = !!selected && words.length === selected.operands.length
    && words.every(word => /^[0-9a-f]{2}$/i.test(word));
  const changed = !!selected && hex(words.map(word => parseInt(word, 16))) !== hex(selected.operands);
  const callTargets = region ? instructions.map(entry => ({
    address: ((region.pc_offset + entry.offset) & 0x7fff) | 0x8000,
    label: `+${entry.offset.toString(16).padStart(4, '0')} ${operation(entry.opcode)}`,
  })) : [];
  const originalTarget = selected?.opcode === 0x44
    ? selected.operands[0] | (selected.operands[1] << 8) : null;
  const currentTarget = valid && selected?.opcode === 0x44
    ? parseInt(words[0], 16) | (parseInt(words[1], 16) << 8) : '';

  async function save() {
    if (!region || !selected || !valid || !changed || busy) return;
    const token = request.current;
    setBusy(true);
    setError('');
    try {
      const result = await invoke<Instruction[]>('update_ai_script_operands', {
        pcOffset: region.pc_offset,
        expectedBytes: instructions.flatMap(entry => [entry.opcode, ...entry.operands]),
        instructionOffset: selected.offset,
        operands: words.map(word => parseInt(word, 16)),
      });
      if (request.current === token) setInstructions(result);
      await Promise.all([refreshUndoState(), refreshPendingWrites()]);
    } catch (reason) {
      if (request.current === token) setError(String(reason));
    } finally {
      if (request.current === token) setBusy(false);
    }
  }

  return <details className="border border-slate-700 rounded p-4 mb-4">
    <summary className="font-semibold cursor-pointer">AI instruction editor (experimental)</summary>
    <p className="text-sm text-amber-300 my-3">USA primary scripts only. Unknown operands are not validated for gameplay.
      Subroutines may be shared. Edits preserve instruction sizes and are undoable; test changes in the emulator.</p>
    <label>Fighter script <select aria-label="Fighter script" value={regionId} disabled={busy || !regions.length}
      onChange={event => setRegionId(event.target.value)} className="bg-slate-800 p-2 ml-2">
      {regions.map(entry => <option key={entry.id} value={entry.id}>{entry.fighter}</option>)}
    </select></label>
    <button disabled={busy || !region} onClick={() => setReload(value => value + 1)} className="ml-3">Reload</button>
    {error && <p role="alert" className="text-red-400 my-2">{error}</p>}
    {busy && <p role="status">Loading or saving script...</p>}
    {region && <p className="text-sm text-slate-400 my-2">PC ${region.pc_offset.toString(16)}; {region.length} bytes</p>}
    <div className="max-h-64 overflow-auto">
      {instructions.map(entry => <button key={entry.offset} disabled={busy}
        aria-pressed={selectedOffset === entry.offset}
        className={`block w-full text-left font-mono p-1 ${selectedOffset === entry.offset ? 'bg-slate-700' : ''}`}
        onClick={() => { setSelectedOffset(entry.offset); setDraft(hex(entry.operands)); setWaitFrames(String(entry.operands[0] ?? '')); }}>
        +{entry.offset.toString(16).padStart(4, '0')} {operation(entry.opcode)} [{hex(entry.operands)}]
      </button>)}
    </div>
    {selected && <div className="mt-3">
      {selected.opcode === 0x2a ? <label>Wait frames <input aria-label="Wait frames" type="number" min={0} max={255} step={1}
        value={waitFrames} disabled={busy} onChange={event => setWaitFrames(event.target.value)}
        className="bg-slate-800 p-2 font-mono" /></label> : selected.opcode === 0x44 ?
        <label>Call destination <select aria-label="Call destination" value={currentTarget} disabled={busy}
          className="bg-slate-800 p-2 font-mono" onChange={event => {
            const address = Number(event.target.value);
            setDraft(hex([address & 255, address >> 8]));
          }}>
          {originalTarget !== null && !callTargets.some(target => target.address === originalTarget) &&
            <option value={originalTarget}>Existing external target ${originalTarget.toString(16)}</option>}
          {callTargets.map(target => <option key={target.address} value={target.address}>
            ${target.address.toString(16)} {target.label}
          </option>)}
        </select></label> : <label>Operands (hex bytes) <input aria-label="Operands (hex bytes)" value={draft}
        disabled={busy || !selected.operands.length} onChange={event => setDraft(event.target.value)}
        className="bg-slate-800 p-2 font-mono" /></label>}
      <button disabled={busy || !valid || !changed} onClick={() => void save()} className="ml-3 disabled:opacity-40">Apply operands</button>
      {!valid && <p role="alert">{selected.opcode === 0x2a ? 'Enter a whole frame count from 0 to 255.' : `Enter exactly ${selected.operands.length} two-digit hex bytes.`}</p>}
    </div>}
  </details>;
}
