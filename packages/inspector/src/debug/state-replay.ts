/**
 * State replay — reproduce bugs by replaying a sequence of state transitions.
 */

export interface ReplayStep {
  machineId: string;
  event: string;
  delayMs: number;
}

export class StateReplay {
  private steps: ReplayStep[] = [];
  private playing = false;

  /** Record a step for replay. */
  addStep(machineId: string, event: string, delayMs: number = 100): void {
    this.steps.push({ machineId, event, delayMs });
  }

  /** Play back all recorded steps. */
  async play(
    onStep: (step: ReplayStep, index: number) => void
  ): Promise<void> {
    this.playing = true;

    for (let i = 0; i < this.steps.length; i++) {
      if (!this.playing) break;
      const step = this.steps[i];
      onStep(step, i);
      await new Promise((r) => setTimeout(r, step.delayMs));
    }

    this.playing = false;
  }

  /** Stop playback. */
  stop(): void {
    this.playing = false;
  }

  /** Clear all recorded steps. */
  clear(): void {
    this.steps = [];
  }

  /** Get step count. */
  get length(): number {
    return this.steps.length;
  }

  /** Export as JSON for sharing. */
  export(): string {
    return JSON.stringify(this.steps, null, 2);
  }

  /** Import from JSON. */
  import(json: string): void {
    this.steps = JSON.parse(json);
  }
}
