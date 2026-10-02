import { GloveIcon, PaintIcon, PlayIcon, FolderIcon } from "./icons";
import "./Usability.css";

interface WelcomeWorkspaceProps {
  isDesktopRuntime: boolean;
  onOpenRom: () => void;
}

/**
 * An original boxing-ring illustration drawn for this app. It contains no
 * game artwork, characters or logos.
 */
function RingArt(): React.ReactElement {
  return (
    <svg className="welcome-ring-art" viewBox="0 0 320 240" role="img" aria-label="A boxing ring under a spotlight">
      <defs>
        <linearGradient id="ring-spot" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#ffffff" stopOpacity="0.55" />
          <stop offset="1" stopColor="#ffffff" stopOpacity="0" />
        </linearGradient>
      </defs>
      <polygon points="130,0 190,0 262,150 58,150" fill="url(#ring-spot)" />
      {/* canvas */}
      <polygon points="52,150 268,150 306,214 14,214" fill="#f4f1fa" />
      <polygon points="14,214 306,214 306,232 14,232" fill="#2a63e0" />
      <polygon points="14,214 306,214 306,220 14,220" fill="#173c91" opacity="0.45" />
      {/* centre star */}
      <path
        d="m160 166 5.300 10.600 11.900 1.500-8.800 8.100 2.300 11.600L160 192.200l-10.700 5.600 2.300-11.600-8.800-8.100 11.900-1.500Z"
        fill="#e5121d"
      />
      {/* back ropes */}
      <g stroke="#ffffff" strokeWidth="3" strokeLinecap="round">
        <line x1="60" y1="92" x2="260" y2="92" stroke="#e5121d" />
        <line x1="58" y1="110" x2="262" y2="110" />
        <line x1="55" y1="128" x2="265" y2="128" stroke="#2a63e0" />
      </g>
      {/* posts */}
      <rect x="50" y="80" width="12" height="74" rx="3" fill="#e5121d" />
      <rect x="258" y="80" width="12" height="74" rx="3" fill="#2a63e0" />
      <rect x="8" y="124" width="14" height="94" rx="3" fill="#2a63e0" />
      <rect x="298" y="124" width="14" height="94" rx="3" fill="#e5121d" />
      {/* side ropes */}
      <g strokeWidth="3.500" strokeLinecap="round">
        <line x1="56" y1="92" x2="15" y2="138" stroke="#e5121d" />
        <line x1="56" y1="110" x2="15" y2="162" stroke="#ffffff" />
        <line x1="56" y1="128" x2="15" y2="186" stroke="#2a63e0" />
        <line x1="264" y1="92" x2="305" y2="138" stroke="#e5121d" />
        <line x1="264" y1="110" x2="305" y2="162" stroke="#ffffff" />
        <line x1="264" y1="128" x2="305" y2="186" stroke="#2a63e0" />
      </g>
      {/* front ropes */}
      <g strokeWidth="4" strokeLinecap="round">
        <line x1="15" y1="138" x2="305" y2="138" stroke="#e5121d" />
        <line x1="15" y1="162" x2="305" y2="162" stroke="#ffffff" />
        <line x1="15" y1="186" x2="305" y2="186" stroke="#2a63e0" />
      </g>
    </svg>
  );
}

export function WelcomeWorkspace({
  isDesktopRuntime,
  onOpenRom,
}: WelcomeWorkspaceProps): React.ReactElement {
  return (
    <section className="welcome-workspace" aria-labelledby="welcome-title">
      <div className="welcome-hero">
        <div className="welcome-hero-copy">
          <p className="eyebrow">Start here</p>
          <h2 id="welcome-title">Make the game your own.</h2>
          <p className="welcome-lead">
            Open your game, change how the boxers look, then play it right away.
            Your original game file is never changed.
          </p>
          <button
            type="button"
            className="welcome-primary-action"
            onClick={onOpenRom}
            disabled={!isDesktopRuntime}
          >
            <span className="welcome-action-mark" aria-hidden="true">
              <GloveIcon size={30} />
            </span>
            <span className="welcome-action-text">
              Open My ROM
              <small>Your Super Punch-Out!! game file (.sfc or .smc)</small>
            </span>
          </button>
        </div>
        <RingArt />
      </div>

      <div className="welcome-promise-grid">
        <article>
          <span className="welcome-step-icon pad-blue" aria-hidden="true"><FolderIcon /></span>
          <div>
            <p className="eyebrow">Step 1</p>
            <h3>Open</h3>
            <p>Pick your own game file on this computer. No game is included with the editor.</p>
          </div>
        </article>
        <article>
          <span className="welcome-step-icon pad-red" aria-hidden="true"><PaintIcon /></span>
          <div>
            <p className="eyebrow">Step 2</p>
            <h3>Change</h3>
            <p>Pick a boxer and try a new color. Made a mistake? Undo fixes it.</p>
          </div>
        </article>
        <article>
          <span className="welcome-step-icon pad-green" aria-hidden="true"><PlayIcon /></span>
          <div>
            <p className="eyebrow">Step 3</p>
            <h3>Play</h3>
            <p>Play your version, then save it as a new file. Your original stays safe.</p>
          </div>
        </article>
      </div>

      <div className="welcome-safety-note">
        <strong>Your ROM stays on this computer.</strong>
        <span>
          Never send your ROM or save files to anyone when reporting a problem.
          The Tester Checklist in the sidebar makes a safe report for you.
        </span>
      </div>

      <p className="welcome-legal">
        This is an unofficial, fan-made editor. It is not affiliated with, sponsored by, or endorsed by Nintendo.
        Super Punch-Out!! is a trademark of Nintendo. You must supply your own legally obtained copy of the game.
      </p>
    </section>
  );
}
