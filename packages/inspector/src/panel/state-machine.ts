/**
 * State machine visualizer — shows states as a graph with live
 * current-state highlighting and transition history.
 */

export interface StateMachineInfo {
  id: string;
  name: string;
  states: string[];
  transitions: Array<{ event: string; from: string; to: string }>;
  currentState: string;
}

export interface TransitionRecord {
  timestamp: number;
  machineId: string;
  event: string;
  from: string;
  to: string;
}

export class StateMachineVisualizer {
  private history: TransitionRecord[] = [];
  private container: HTMLElement | null = null;
  private maxHistory = 50;

  /** Record a state transition. */
  recordTransition(machineId: string, event: string, from: string, to: string): void {
    this.history.push({
      timestamp: Date.now(),
      machineId,
      event,
      from,
      to,
    });
    if (this.history.length > this.maxHistory) {
      this.history.shift();
    }
    this.refreshHistory();
  }

  /** Render the state machine visualizer into a container. */
  render(machines: StateMachineInfo[], container: HTMLElement): void {
    this.container = container;

    container.innerHTML = `
      <div style="margin-bottom:12px">
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:8px">
          State Machines (${machines.length})
        </div>
        ${machines.map((sm) => this.renderMachine(sm)).join("")}
      </div>
      <div style="margin-bottom:12px">
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:8px">
          Transition History
        </div>
        <div id="voce-sm-history" style="max-height:200px;overflow-y:auto;font-family:monospace;font-size:11px">
          ${this.renderHistory()}
        </div>
      </div>
    `;
  }

  private renderMachine(sm: StateMachineInfo): string {
    const stateNodes = sm.states
      .map((s) => {
        const isCurrent = s === sm.currentState;
        const bg = isCurrent ? "rgba(68,255,68,0.2)" : "rgba(255,255,255,0.05)";
        const border = isCurrent ? "#44ff44" : "#444";
        const color = isCurrent ? "#44ff44" : "#aaa";
        return `<span style="display:inline-block;padding:4px 10px;border:1px solid ${border};border-radius:4px;margin:2px;background:${bg};color:${color};font-size:11px">${s}</span>`;
      })
      .join("");

    const transitions = sm.transitions
      .map(
        (t) =>
          `<div style="color:#666;font-size:10px;padding:1px 0">
            ${t.from} <span style="color:#4488ff">→</span> ${t.to} <span style="color:#888">(${t.event})</span>
          </div>`
      )
      .join("");

    return `
      <div style="background:rgba(255,255,255,0.03);border:1px solid #333;border-radius:6px;padding:10px;margin-bottom:8px">
        <div style="font-weight:bold;color:#ddd;margin-bottom:6px">${sm.name || sm.id}</div>
        <div style="margin-bottom:6px">${stateNodes}</div>
        <details style="color:#888">
          <summary style="cursor:pointer;font-size:11px">${sm.transitions.length} transitions</summary>
          <div style="padding-top:4px">${transitions}</div>
        </details>
      </div>
    `;
  }

  private renderHistory(): string {
    if (this.history.length === 0) {
      return '<div style="color:#555">No transitions recorded yet</div>';
    }
    return this.history
      .slice()
      .reverse()
      .map((t) => {
        const time = new Date(t.timestamp).toLocaleTimeString();
        return `<div style="padding:2px 0;border-bottom:1px solid #222">
          <span style="color:#555">${time}</span>
          <span style="color:#888">${t.machineId}:</span>
          <span style="color:#aaa">${t.from}</span>
          <span style="color:#4488ff">→</span>
          <span style="color:#aaa">${t.to}</span>
          <span style="color:#666">(${t.event})</span>
        </div>`;
      })
      .join("");
  }

  private refreshHistory(): void {
    const el = this.container?.querySelector("#voce-sm-history");
    if (el) {
      el.innerHTML = this.renderHistory();
    }
  }
}
