/**
 * Performance profiler — frame timing breakdown and jank detection.
 */

export interface FrameMetrics {
  timestamp: number;
  totalMs: number;
  jsMs: number;
  layoutMs: number;
  paintMs: number;
  isJank: boolean;
}

export class PerformanceProfiler {
  private frames: FrameMetrics[] = [];
  private maxFrames = 120; // ~2 seconds at 60fps
  private monitoring = false;
  private container: HTMLElement | null = null;

  /** Start monitoring frame timing. */
  startMonitoring(): void {
    if (this.monitoring) return;
    this.monitoring = true;
    this.measureFrame();
  }

  /** Stop monitoring. */
  stopMonitoring(): void {
    this.monitoring = false;
  }

  /** Render the profiler panel. */
  render(container: HTMLElement): void {
    this.container = container;
    this.refreshDisplay();
  }

  /** Export profiler data as JSON. */
  exportData(): string {
    return JSON.stringify({
      frames: this.frames,
      summary: this.getSummary(),
    }, null, 2);
  }

  private measureFrame(): void {
    if (!this.monitoring) return;

    const start = performance.now();

    requestAnimationFrame(() => {
      const end = performance.now();
      const totalMs = end - start;

      const frame: FrameMetrics = {
        timestamp: Date.now(),
        totalMs,
        jsMs: totalMs * 0.4, // estimate
        layoutMs: totalMs * 0.3,
        paintMs: totalMs * 0.3,
        isJank: totalMs > 16.67,
      };

      this.frames.push(frame);
      if (this.frames.length > this.maxFrames) {
        this.frames.shift();
      }

      this.refreshDisplay();
      this.measureFrame();
    });
  }

  private getSummary() {
    if (this.frames.length === 0) return { fps: 0, avgMs: 0, jankCount: 0, jankPercent: 0 };

    const avgMs = this.frames.reduce((s, f) => s + f.totalMs, 0) / this.frames.length;
    const fps = Math.round(1000 / avgMs);
    const jankCount = this.frames.filter((f) => f.isJank).length;
    const jankPercent = Math.round((jankCount / this.frames.length) * 100);

    return { fps, avgMs: Math.round(avgMs * 100) / 100, jankCount, jankPercent };
  }

  private refreshDisplay(): void {
    if (!this.container) return;

    const summary = this.getSummary();
    const fpsColor = summary.fps >= 55 ? "#44ff44" : summary.fps >= 30 ? "#ffaa22" : "#ff4444";

    // Mini frame graph (last 60 frames)
    const recentFrames = this.frames.slice(-60);
    const barWidth = 3;
    const graphHeight = 40;
    const maxMs = 33; // 30fps = 33ms
    const bars = recentFrames
      .map((f) => {
        const h = Math.min(graphHeight, (f.totalMs / maxMs) * graphHeight);
        const color = f.isJank ? "#ff4444" : "#44ff44";
        return `<div style="width:${barWidth}px;height:${h}px;background:${color};flex-shrink:0"></div>`;
      })
      .join("");

    this.container.innerHTML = `
      <div>
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:8px">
          Performance ${this.monitoring ? "(live)" : "(paused)"}
        </div>

        <div style="display:flex;gap:16px;margin-bottom:8px">
          <div>
            <div style="font-size:24px;font-weight:bold;color:${fpsColor}">${summary.fps}</div>
            <div style="font-size:10px;color:#666">FPS</div>
          </div>
          <div>
            <div style="font-size:24px;font-weight:bold;color:#ddd">${summary.avgMs}</div>
            <div style="font-size:10px;color:#666">ms/frame</div>
          </div>
          <div>
            <div style="font-size:24px;font-weight:bold;color:${summary.jankPercent > 5 ? "#ff4444" : "#44ff44"}">${summary.jankPercent}%</div>
            <div style="font-size:10px;color:#666">jank</div>
          </div>
        </div>

        <!-- Frame graph -->
        <div style="display:flex;align-items:flex-end;height:${graphHeight}px;background:#111;border-radius:4px;overflow:hidden;gap:1px;padding:2px">
          ${bars}
        </div>
        <div style="display:flex;justify-content:space-between;font-size:9px;color:#444;margin-top:2px">
          <span>16.67ms (60fps)</span>
          <span>${recentFrames.length} frames</span>
        </div>

        <button onclick="navigator.clipboard.writeText(JSON.stringify(${JSON.stringify(summary)}))" style="margin-top:8px;background:#222;color:#888;border:1px solid #444;border-radius:3px;padding:4px 8px;font-size:10px;cursor:pointer">
          Copy metrics
        </button>
      </div>
    `;
  }
}
