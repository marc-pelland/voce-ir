/**
 * Animation timeline — scrubable horizontal timeline for all
 * active animations with pause, play, step, and speed control.
 */

export interface AnimationInfo {
  id: string;
  targetNodeId: string;
  properties: string[];
  durationMs: number;
  easing: string;
  hasReducedMotion: boolean;
  state: "idle" | "running" | "paused" | "finished";
}

export class AnimationTimeline {
  private animations: AnimationInfo[] = [];
  private container: HTMLElement | null = null;
  private paused = false;
  private speed = 1.0;
  private scrubPosition = 0; // 0.0 to 1.0

  /** Render the timeline into a container. */
  render(animations: AnimationInfo[], container: HTMLElement): void {
    this.animations = animations;
    this.container = container;

    container.innerHTML = `
      <div style="margin-bottom:12px">
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:8px">
          Animations (${animations.length})
        </div>

        <!-- Timeline controls -->
        <div style="display:flex;gap:8px;align-items:center;margin-bottom:8px">
          <button id="voce-anim-play" style="${this.buttonStyle()}" title="Play/Pause (Space)">
            ${this.paused ? "▶" : "⏸"}
          </button>
          <button id="voce-anim-step-back" style="${this.buttonStyle()}" title="Step Back (←)">⏪</button>
          <button id="voce-anim-step-fwd" style="${this.buttonStyle()}" title="Step Forward (→)">⏩</button>
          <select id="voce-anim-speed" style="background:#222;color:#aaa;border:1px solid #444;border-radius:3px;padding:2px 4px;font-size:11px">
            <option value="0.25" ${this.speed === 0.25 ? "selected" : ""}>0.25x</option>
            <option value="0.5" ${this.speed === 0.5 ? "selected" : ""}>0.5x</option>
            <option value="1" ${this.speed === 1 ? "selected" : ""}>1x</option>
            <option value="2" ${this.speed === 2 ? "selected" : ""}>2x</option>
          </select>
        </div>

        <!-- Scrub bar -->
        <div style="position:relative;height:24px;background:#1a1a2e;border:1px solid #333;border-radius:4px;margin-bottom:12px;cursor:pointer" id="voce-anim-scrub">
          <div style="position:absolute;top:0;left:0;height:100%;width:${this.scrubPosition * 100}%;background:rgba(68,136,255,0.3);border-radius:3px"></div>
          <div style="position:absolute;top:0;left:${this.scrubPosition * 100}%;width:2px;height:100%;background:#4488ff"></div>
        </div>

        <!-- Per-animation breakdown -->
        ${animations.map((a) => this.renderAnimation(a)).join("")}
      </div>
    `;

    this.setupControls(container);
  }

  private renderAnimation(anim: AnimationInfo): string {
    const stateColor =
      anim.state === "running" ? "#44ff44" :
      anim.state === "paused" ? "#ffaa22" :
      anim.state === "finished" ? "#888" : "#555";

    return `
      <div style="background:rgba(255,255,255,0.03);border:1px solid #333;border-radius:4px;padding:8px;margin-bottom:4px">
        <div style="display:flex;justify-content:space-between;align-items:center">
          <span style="color:#ddd;font-size:12px">${anim.id}</span>
          <span style="color:${stateColor};font-size:10px">${anim.state}</span>
        </div>
        <div style="color:#666;font-size:10px;margin-top:4px">
          Target: ${anim.targetNodeId} | ${anim.durationMs}ms | ${anim.easing}
          ${anim.hasReducedMotion ? ' | <span style="color:#44aa44">♿ reduced-motion</span>' : ""}
        </div>
        <div style="color:#555;font-size:10px">
          Properties: ${anim.properties.join(", ")}
        </div>
      </div>
    `;
  }

  private setupControls(container: HTMLElement): void {
    container.querySelector("#voce-anim-play")?.addEventListener("click", () => {
      this.paused = !this.paused;
      this.render(this.animations, this.container!);
    });

    container.querySelector("#voce-anim-speed")?.addEventListener("change", (e) => {
      this.speed = parseFloat((e.target as HTMLSelectElement).value);
    });

    container.querySelector("#voce-anim-scrub")?.addEventListener("click", (ev) => {
      const e = ev as MouseEvent;
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      this.scrubPosition = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
      this.render(this.animations, this.container!);
    });

    // Keyboard shortcuts
    document.addEventListener("keydown", (e) => {
      if (e.key === " " && e.target === document.body) {
        e.preventDefault();
        this.paused = !this.paused;
        this.render(this.animations, this.container!);
      }
      if (e.key === "ArrowRight") {
        this.scrubPosition = Math.min(1, this.scrubPosition + 0.02);
        this.render(this.animations, this.container!);
      }
      if (e.key === "ArrowLeft") {
        this.scrubPosition = Math.max(0, this.scrubPosition - 0.02);
        this.render(this.animations, this.container!);
      }
    });
  }

  private buttonStyle(): string {
    return "background:#222;color:#aaa;border:1px solid #444;border-radius:3px;padding:4px 8px;cursor:pointer;font-size:12px";
  }
}
