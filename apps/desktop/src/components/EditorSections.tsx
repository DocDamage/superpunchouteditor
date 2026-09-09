import { useId, useRef, useState, type KeyboardEvent, type ReactNode } from "react";

export type EditorSectionKey = "colors" | "sprites" | "assets" | "export";
const SECTIONS: { key: EditorSectionKey; label: string; hint: string; marker: string }[] = [
  { key: "colors", label: "Colors", hint: "Start with a palette", marker: "01" },
  { key: "sprites", label: "Sprites", hint: "Preview & edit pixels", marker: "02" },
  { key: "assets", label: "Assets", hint: "Import & export artwork", marker: "03" },
  { key: "export", label: "Export", hint: "Save a playable copy", marker: "04" },
];

interface Props { panels: Record<EditorSectionKey, ReactNode> }

/** Manual-activation tabs; visited panels stay mounted so local drafts survive. */
export function EditorSections({ panels }: Props) {
  const id = useId();
  const [active, setActive] = useState<EditorSectionKey>("colors");
  const [focused, setFocused] = useState<EditorSectionKey>("colors");
  const [visited, setVisited] = useState<ReadonlySet<EditorSectionKey>>(new Set(["colors"]));
  const buttons = useRef<Array<HTMLButtonElement | null>>([]);
  const activate = (key: EditorSectionKey) => {
    setActive(key);
    setFocused(key);
    setVisited((current) => current.has(key) ? current : new Set([...current, key]));
  };
  const handleKey = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    let target: number;
    if (event.key === "ArrowRight") target = (index + 1) % SECTIONS.length;
    else if (event.key === "ArrowLeft") target = (index + SECTIONS.length - 1) % SECTIONS.length;
    else if (event.key === "Home") target = 0;
    else if (event.key === "End") target = SECTIONS.length - 1;
    else return;
    event.preventDefault();
    setFocused(SECTIONS[target].key);
    buttons.current[target]?.focus();
  };
  return (
    <div className="club-sections">
      <div role="tablist" aria-label="Boxer editing tools" className="club-tool-tabs"
        onBlur={(event) => {
          if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setFocused(active);
        }}>
        {SECTIONS.map((section, index) => (
          <button type="button" key={section.key} role="tab" id={`${id}-tab-${section.key}`}
            aria-controls={`${id}-panel-${section.key}`} aria-selected={active === section.key}
            tabIndex={focused === section.key ? 0 : -1} className="club-tool-tab"
            ref={(node) => { buttons.current[index] = node; }}
            onKeyDown={(event) => handleKey(event, index)} onClick={() => activate(section.key)}>
            <span className={`club-tool-marker club-marker-${section.key}`} aria-hidden="true">{section.marker}</span>
            <span><strong>{section.label}</strong><small>{section.hint}</small></span>
          </button>
        ))}
      </div>
      {SECTIONS.map(({ key }) => (
        <section key={key} role="tabpanel" id={`${id}-panel-${key}`} aria-labelledby={`${id}-tab-${key}`}
          tabIndex={0} hidden={active !== key} className="club-tool-panel">
          {visited.has(key) ? panels[key] : null}
        </section>
      ))}
    </div>
  );
}
