import { useId, useMemo, useState } from "react";

interface Props {
  boxers: Array<{ key: string; name: string }>;
  selectedKey?: string | null;
  portraits: Record<string, string>;
  onSelect: (key: string) => void;
  compact?: boolean;
  busy?: boolean;
}

export function BoxerPicker({ boxers, selectedKey, portraits, onSelect, compact = false, busy = false }: Props) {
  const id = useId();
  const [query, setQuery] = useState("");
  const filtered = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    return boxers.filter((boxer) => boxer.name.toLocaleLowerCase().includes(needle));
  }, [boxers, query]);
  return (
    <section className={`club-boxer-picker ${compact ? "club-picker-compact" : ""}`} aria-label="Choose boxer" aria-busy={busy}>
      <label htmlFor={`${id}-search`} className="guided-section-title">Find your boxer</label>
      <div className="club-search-row">
        <input id={`${id}-search`} type="search" value={query} placeholder="Search by name…" autoComplete="off"
          aria-controls={`${id}-results`} onChange={(event) => setQuery(event.target.value)} />
        {query && <button type="button" className="secondary" onClick={() => setQuery("")} aria-label="Clear boxer search">Clear</button>}
      </div>
      <p className="club-picker-count" role="status">
        {busy ? "Loading boxer…" : `${filtered.length} of ${boxers.length} boxers`}
      </p>
      <ul id={`${id}-results`} className="club-boxer-grid">
        {filtered.map((boxer, index) => (
          <li key={boxer.key}>
            <button type="button" className="club-boxer-card" aria-pressed={selectedKey === boxer.key}
              disabled={busy} onClick={() => onSelect(boxer.key)}>
              {portraits[boxer.key] ? <img src={portraits[boxer.key]} alt="" />
                : <span className="club-boxer-fallback" aria-hidden="true">{String(index + 1).padStart(2, "0")}</span>}
              <span>{boxer.name}</span>
              {selectedKey === boxer.key && <small className="club-selected-tag">Selected</small>}
            </button>
          </li>
        ))}
      </ul>
      {filtered.length === 0 && <p className="club-search-empty">{boxers.length === 0 ? "No boxer data is available. Reopen your ROM and check its region." : <>No boxers match “{query.trim()}”. Clear the search to see everyone.</>}</p>}
    </section>
  );
}
