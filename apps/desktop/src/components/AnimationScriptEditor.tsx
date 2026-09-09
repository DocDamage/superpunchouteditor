import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';

interface Instruction { offset: number; opcode: number; operands: number[] }
interface Region { id: string; label: string }
interface TimingTrace {
  events: {
    instruction_offset: number; pose: number | null; updates: number;
    facing_request: 'left' | 'right' | null;
    position_request: { x_unmirrored: number; x_mirrored: number; y: number } | null;
  }[];
  stop: { reason: string; offset?: number; address?: number };
}
interface Reference {
  source_region: string; target_region: string | null;
  reference: { instruction_offset: number; target_address: number; resolution: string };
}

export function AnimationScriptEditor() {
  const { romSha1, undoStack, refreshUndoState, refreshPendingWrites } = useStore();
  const [instructions, setInstructions] = useState<Instruction[]>([]);
  const [references, setReferences] = useState<Reference[]>([]);
  const [timing, setTiming] = useState<TimingTrace | null>(null);
  const [regions, setRegions] = useState<Region[]>([]);
  const [regionId, setRegionId] = useState('');
  const [offset, setOffset] = useState<number | null>(null);
  const [duration, setDuration] = useState('');
  const [staged, setStaged] = useState<Record<number, number>>({});
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [reload, setReload] = useState(0);
  const request = useRef(0);

  useEffect(() => {
    let cancelled = false;
    setRegions([]);
    setRegionId('');
    invoke<Region[]>('get_usa_animation_regions').then(result => {
      if (!cancelled) { setRegions(result); setRegionId(result[0]?.id ?? ''); }
    }).catch(reason => { if (!cancelled) setError(String(reason)); });
    return () => { cancelled = true; };
  }, [romSha1]);

  useEffect(() => {
    const token = ++request.current;
    setInstructions([]);
    setReferences([]);
    setTiming(null);
    setOffset(null);
    setDuration('');
    setStaged({});
    setError('');
    if (!regionId) { setBusy(false); return; }
    setBusy(true);
    invoke<{ instructions: Instruction[]; references: Reference[]; timing: TimingTrace }>('read_reference_animation', { regionId })
      .then(snapshot => { if (token === request.current) { setInstructions(snapshot.instructions); setReferences(snapshot.references); setTiming(snapshot.timing); } })
      .catch(reason => { if (token === request.current) setError(String(reason)); })
      .finally(() => { if (token === request.current) setBusy(false); });
    return () => { request.current++; };
  }, [romSha1, undoStack, reload, regionId]);

  const frames = instructions.filter(i => i.opcode === 0x20 && i.operands.length === 2);
  const selected = frames.find(i => i.offset === offset);
  const valid = /^\d+$/.test(duration) && Number(duration) <= 255;
  const changed = selected && valid && Number(duration) !== selected.operands[1];

  async function commit(batch: boolean) {
    if (busy || (batch ? !Object.keys(staged).length : !selected || !changed)) return;
    const token = request.current;
    setBusy(true);
    setError('');
    try {
      const common = { regionId, expectedBytes: instructions.flatMap(i => [i.opcode, ...i.operands]) };
      if (batch) {
        await invoke('update_animation_durations', { ...common,
          edits: Object.entries(staged).map(([key, value]) => ({ instruction_offset: Number(key), duration: value })),
        });
      } else {
        await invoke('update_reference_animation_duration', { ...common,
          instructionOffset: selected!.offset, duration: Number(duration) });
      }
      await Promise.all([refreshUndoState(), refreshPendingWrites()]);
      if (token === request.current) setReload(value => value + 1);
    } catch (reason) {
      if (token === request.current) setError(String(reason));
    } finally {
      if (token === request.current) setBusy(false);
    }
  }

  return <details>
    <summary>Animation bytecode duration editor (experimental)</summary>
    <p>Verified USA intervals only. These are decoded pose instructions,
      not reconstructed animation playback. Other sequences and hitboxes are not editable here.</p>
    {error && <p role="alert">{error}</p>}
    <label>Animation sequence
      <select value={regionId} disabled={busy || !regions.length} onChange={event => setRegionId(event.target.value)}>
        {!regions.length && <option value="">No verified sequences available</option>}
        {regions.map(region => <option key={region.id} value={region.id}>{region.label}</option>)}
      </select>
    </label>
    <label>Pose instruction
      <select disabled={busy} value={offset ?? ''} onChange={event => {
        const frame = frames.find(i => i.offset === Number(event.target.value));
        setOffset(frame?.offset ?? null);
        setDuration(frame ? String(staged[frame.offset] ?? frame.operands[1]) : '');
      }}>
        <option value="" disabled>Select a pose instruction</option>
        {frames.map(frame => <option key={frame.offset} value={frame.offset}>
          +{frame.offset.toString(16)}: pose {frame.operands[0]}, duration {frame.operands[1]}
        </option>)}
      </select>
    </label>
    <label>Duration byte (0-255)
      <input inputMode="numeric" value={duration} disabled={!selected || busy}
        onChange={event => setDuration(event.target.value)} />
    </label>
    {selected && valid && <p>
      Delay counter expires after {Number(duration) === 0 ? 256 : Number(duration)} animation updates
      if uninterrupted. A zero byte wraps to 256 updates; it is not an instant frame.
    </p>}
    <button disabled={!changed || busy || Object.keys(staged).length > 0} onClick={() => commit(false)}>Apply duration</button>
    <button disabled={!selected || !valid || busy} onClick={() => {
      if (!selected) return;
      setStaged(previous => {
        const next = { ...previous };
        if (Number(duration) === selected.operands[1]) delete next[selected.offset];
        else next[selected.offset] = Number(duration);
        return next;
      });
    }}>Stage duration</button>
    {Object.keys(staged).length > 0 && <section aria-label="Pending duration changes">
      <ul>{Object.entries(staged).map(([key, value]) => <li key={key}>
        +{Number(key).toString(16)}: {frames.find(frame => frame.offset === Number(key))?.operands[1]} to {value}
        <button disabled={busy} aria-label={`Remove staged duration at ${key}`} onClick={() => setStaged(previous => {
          const next = { ...previous }; delete next[Number(key)]; return next;
        })}>Remove</button>
      </li>)}</ul>
      <button disabled={busy} onClick={() => commit(true)}>Apply staged durations</button>
      <p>Applies all staged changes in one Undo step. Reload or history changes discard this draft.</p>
    </section>}
    <button disabled={busy} onClick={() => setReload(value => value + 1)}>Reload animation</button>
    <h4>Script references</h4>
    {timing && <section aria-label="Verified timing trace">
      <h4>Partial timing trace</h4>
      <p>Stopped: {timing.stop.reason.replace(/_/g, ' ')}
        {timing.stop.offset !== undefined && ` at +${timing.stop.offset.toString(16)}`}
        {timing.stop.address !== undefined && ` at $${timing.stop.address.toString(16)}`}.
        This is not complete playback; unknown operations and external targets stop analysis.</p>
      <ol>{timing.events.map((event, index) => <li key={index}>
        +{event.instruction_offset.toString(16)}: pose {event.pose ?? 'unknown'}, {event.updates} updates
        {event.facing_request && `, facing request ${event.facing_request} (runtime mirroring not resolved)`}
        {event.position_request && `, position request X ${event.position_request.x_unmirrored} or ${event.position_request.x_mirrored}, Y ${event.position_request.y} (runtime mirroring not resolved)`}
      </li>)}</ol>
    </section>}
    <p>References show possible destinations, not execution order or branch conditions.</p>
    <ul>{references.filter(entry => entry.source_region === regionId).map((entry, index) =>
      <li key={index}>
        +{entry.reference.instruction_offset.toString(16)} to ${entry.reference.target_address.toString(16)}:
        {' '}{entry.reference.resolution.replace(/_/g, ' ')}
        {entry.target_region && entry.reference.resolution === 'instruction' && (
          <button disabled={busy} onClick={() => setRegionId(entry.target_region!)}>
            Open {regions.find(region => region.id === entry.target_region)?.label ?? entry.target_region}
          </button>
        )}
      </li>)}</ul>
  </details>;
}
