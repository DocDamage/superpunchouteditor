import "./Usability.css";

interface WelcomeWorkspaceProps {
  isDesktopRuntime: boolean;
  onOpenRom: () => void;
  onOpenProject?: () => void;
  onHelp?: () => void;
  busy?: boolean;
}

export function WelcomeWorkspace({ isDesktopRuntime, onOpenRom, onOpenProject, onHelp, busy = false }: WelcomeWorkspaceProps) {
  return (
    <section className="welcome-workspace club-welcome" aria-labelledby="welcome-title">
      <div className="welcome-hero club-welcome-hero">
        <div className="club-hero-topline"><span className="club-badge">Your game. Your corner.</span><span className="club-face-buttons" aria-hidden="true"><i /><i /><i /><i /></span></div>
        <p className="eyebrow">Super Punch-Out!! Editor</p>
        <h1 id="welcome-title">Big ideas.<br /><span>A whole new fight.</span></h1>
        <p className="welcome-lead">Give your favorite boxer a new look. Start with a color, try your changes in-game, and save something that is yours.</p>
        <div className="club-welcome-actions">
          <button type="button" className="welcome-primary-action" onClick={onOpenRom} disabled={!isDesktopRuntime || busy}>
            <span>{busy ? "Opening…" : "Open My ROM"}</span><small>Your own Super Punch-Out!! .sfc or .smc</small>
          </button>
          {onOpenProject && <button type="button" className="secondary" onClick={onOpenProject} disabled={!isDesktopRuntime || busy}>Continue a project</button>}
        </div>
        {!isDesktopRuntime && <p className="club-runtime-note">You are viewing the interface in a browser. Launch the desktop app to open local ROMs and use the editing tools.</p>}
      </div>
      <div className="welcome-promise-grid club-route-cards">
        <article><span className="welcome-step-number">1</span><div><h2>Pick your boxer</h2><p>Open your own ROM, then choose a familiar face from the character cards.</p></div></article>
        <article><span className="welcome-step-number">2</span><div><h2>Make it yours</h2><p>Start with Colors. Move on to Sprites and Assets when you are ready. Undo lets you experiment.</p></div></article>
        <article><span className="welcome-step-number">3</span><div><h2>Take it to the ring</h2><p>Test Game runs your current revision. Export a new ROM to play, or save a project to keep editing.</p></div></article>
      </div>
      <div className="welcome-safety-note"><strong>Keep your original safe.</strong><span>Use a legally obtained local ROM and export to a new file. No ROM is included. Never attach ROMs or save states to a bug report.</span></div>
      {onHelp && <button type="button" className="club-help-link secondary" onClick={onHelp}>New here? Open the editor guide</button>}
    </section>
  );
}
