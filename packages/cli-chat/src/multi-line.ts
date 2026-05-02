// Multi-line input accumulator. Convention: a trailing `\` continues onto
// the next line. The accumulator is fed one line at a time and returns
// either a partial state ("more input pending") or the full assembled
// message ("submit this").
//
// Multi-line semantics ride on top of Node's readline — no terminal-mode
// surgery required. Works the same on macOS / Linux / Windows Terminal
// as long as the shell forwards line input.

export interface AccumulatorState {
  pending: string[];
}

export function createAccumulator(): AccumulatorState {
  return { pending: [] };
}

export type FeedResult =
  | { kind: "submit"; text: string }
  | { kind: "continue" }
  | { kind: "empty" };

/**
 * Feed one input line. Returns `submit` with the joined message when the
 * line does NOT end with a backslash; `continue` when it does (more lines
 * expected); `empty` for a blank line that's not terminating a multi-line.
 */
export function feedLine(state: AccumulatorState, line: string): FeedResult {
  if (line.endsWith("\\")) {
    state.pending.push(line.slice(0, -1));
    return { kind: "continue" };
  }
  if (state.pending.length === 0 && line.length === 0) {
    return { kind: "empty" };
  }
  const text = [...state.pending, line].join("\n");
  state.pending = [];
  return { kind: "submit", text };
}

/** Reset pending state — called on Ctrl+C to drop a half-typed multi-line. */
export function clearPending(state: AccumulatorState): boolean {
  if (state.pending.length === 0) return false;
  state.pending = [];
  return true;
}
