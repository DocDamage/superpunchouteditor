import { Component, type ErrorInfo, type ReactNode } from "react";

export class PanelBoundary extends Component<{ children: ReactNode }, { message: string | null }> {
  state: { message: string | null } = { message: null };
  static getDerivedStateFromError(error: Error) { return { message: error.message || "Unexpected display error" }; }
  componentDidCatch(error: Error, info: ErrorInfo) { console.error("Panel render failed:", error, info); }
  render() {
    if (this.state.message) return (
      <section className="club-empty-panel" role="alert">
        <p className="eyebrow">Back to your corner</p><h2>This panel needs another try</h2>
        <p>A display error stopped this panel. Your editing session is still open.</p>
        <button type="button" onClick={() => this.setState({ message: null })}>Reload this panel</button>
        <details className="club-details"><summary>Error details for your tester report</summary><pre>{this.state.message}</pre></details>
      </section>
    );
    return this.props.children;
  }
}
