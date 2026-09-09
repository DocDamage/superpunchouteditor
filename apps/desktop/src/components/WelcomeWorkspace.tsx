import "./Usability.css";

interface WelcomeWorkspaceProps {
  isDesktopRuntime: boolean;
  onOpenRom: () => void;
}

export function WelcomeWorkspace({
  isDesktopRuntime,
  onOpenRom,
}: WelcomeWorkspaceProps): React.ReactElement {
  return (
    <section className="welcome-workspace" aria-labelledby="welcome-title">
      <div className="welcome-hero">
        <p className="eyebrow">Start here</p>
        <h2 id="welcome-title">Edit Super Punch-Out!! without fighting the tool.</h2>
        <p className="welcome-lead">
          Open your own ROM, choose what you want to change, test the current revision, then save or export. The editor keeps the original ROM as the immutable base for the session.
        </p>
        <button
          type="button"
          className="welcome-primary-action"
          onClick={onOpenRom}
          disabled={!isDesktopRuntime}
        >
          Open My ROM
          <small>Super Punch-Out!! .sfc or .smc</small>
        </button>
      </div>

      <div className="welcome-promise-grid">
        <article>
          <span className="welcome-step-number">1</span>
          <div>
            <h3>Open</h3>
            <p>Choose a legally obtained ROM stored on this computer. No ROM is included with the editor.</p>
          </div>
        </article>
        <article>
          <span className="welcome-step-number">2</span>
          <div>
            <h3>Edit safely</h3>
            <p>Make a small change first. Undo and Redo work from the canonical edit journal.</p>
          </div>
        </article>
        <article>
          <span className="welcome-step-number">3</span>
          <div>
            <h3>Test, then save</h3>
            <p>Test Game uses the current materialized revision. Save edited ROMs to a new file instead of replacing your source.</p>
          </div>
        </article>
      </div>

      <div className="welcome-safety-note">
        <strong>Your ROM stays local.</strong>
        <span>
          Do not upload ROMs, SRAM, or save states when reporting bugs. Testers can use the Tester Checklist in the left sidebar to generate a safe Markdown report.
        </span>
      </div>
    </section>
  );
}
