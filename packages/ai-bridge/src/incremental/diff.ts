/**
 * IR diff format — JSON Patch (RFC 6902) operations scoped to Voce IR.
 *
 * Patches are minimal, targeted changes that modify specific nodes
 * without regenerating the entire IR.
 */

export type PatchOp = "add" | "remove" | "replace" | "move";

export interface PatchOperation {
  /** JSON Patch operation type. */
  op: PatchOp;
  /** JSON Pointer path to the target (e.g., "/root/children/2/value/content"). */
  path: string;
  /** New value (for add, replace). */
  value?: unknown;
  /** Source path (for move). */
  from?: string;
}

export interface VocePatch {
  /** Human-readable description of the change. */
  description: string;
  /** ISO timestamp. */
  timestamp: string;
  /** The patch operations. */
  operations: PatchOperation[];
}

/**
 * Apply a JSON Patch to an IR JSON object.
 * Returns the patched JSON string.
 *
 * Follows RFC 6902 semantics for add, remove, replace, move.
 */
export function applyPatch(irJson: string, patch: VocePatch): string {
  let ir = JSON.parse(irJson);

  for (const op of patch.operations) {
    ir = applyOperation(ir, op);
  }

  return JSON.stringify(ir, null, 2);
}

/**
 * Generate the inverse of a patch (for undo support).
 * The inverse, when applied, reverts the original patch.
 */
export function invertPatch(
  originalIrJson: string,
  patch: VocePatch
): VocePatch {
  const original = JSON.parse(originalIrJson);
  const inverseOps: PatchOperation[] = [];

  for (const op of [...patch.operations].reverse()) {
    switch (op.op) {
      case "add":
        inverseOps.push({ op: "remove", path: op.path });
        break;
      case "remove": {
        const oldValue = getByPointer(original, op.path);
        inverseOps.push({ op: "add", path: op.path, value: oldValue });
        break;
      }
      case "replace": {
        const oldValue = getByPointer(original, op.path);
        inverseOps.push({ op: "replace", path: op.path, value: oldValue });
        break;
      }
      case "move":
        if (op.from) {
          inverseOps.push({ op: "move", path: op.from, from: op.path });
        }
        break;
    }
  }

  return {
    description: `Undo: ${patch.description}`,
    timestamp: new Date().toISOString(),
    operations: inverseOps,
  };
}

function applyOperation(obj: unknown, op: PatchOperation): unknown {
  const parts = parsePointer(op.path);

  switch (op.op) {
    case "replace":
    case "add":
      return setByParts(obj, parts, op.value);
    case "remove":
      return removeByParts(obj, parts);
    case "move": {
      if (!op.from) return obj;
      const fromParts = parsePointer(op.from);
      const value = getByParts(obj, fromParts);
      const removed = removeByParts(obj, fromParts);
      return setByParts(removed, parts, value);
    }
  }
}

function parsePointer(pointer: string): string[] {
  if (pointer === "" || pointer === "/") return [];
  return pointer
    .split("/")
    .filter((p) => p !== "")
    .map((p) => p.replace(/~1/g, "/").replace(/~0/g, "~"));
}

function getByPointer(obj: unknown, pointer: string): unknown {
  return getByParts(obj, parsePointer(pointer));
}

function getByParts(obj: unknown, parts: string[]): unknown {
  let current: unknown = obj;
  for (const part of parts) {
    if (current === null || current === undefined) return undefined;
    if (Array.isArray(current)) {
      current = current[parseInt(part, 10)];
    } else if (typeof current === "object") {
      current = (current as Record<string, unknown>)[part];
    }
  }
  return current;
}

function setByParts(obj: unknown, parts: string[], value: unknown): unknown {
  if (parts.length === 0) return value;
  const clone = structuredClone(obj);
  let current: unknown = clone;
  for (let i = 0; i < parts.length - 1; i++) {
    if (Array.isArray(current)) {
      current = current[parseInt(parts[i], 10)];
    } else if (typeof current === "object" && current !== null) {
      current = (current as Record<string, unknown>)[parts[i]];
    }
  }
  const lastPart = parts[parts.length - 1];
  if (Array.isArray(current)) {
    if (lastPart === "-") {
      current.push(value);
    } else {
      current[parseInt(lastPart, 10)] = value;
    }
  } else if (typeof current === "object" && current !== null) {
    (current as Record<string, unknown>)[lastPart] = value;
  }
  return clone;
}

function removeByParts(obj: unknown, parts: string[]): unknown {
  if (parts.length === 0) return undefined;
  const clone = structuredClone(obj);
  let current: unknown = clone;
  for (let i = 0; i < parts.length - 1; i++) {
    if (Array.isArray(current)) {
      current = current[parseInt(parts[i], 10)];
    } else if (typeof current === "object" && current !== null) {
      current = (current as Record<string, unknown>)[parts[i]];
    }
  }
  const lastPart = parts[parts.length - 1];
  if (Array.isArray(current)) {
    current.splice(parseInt(lastPart, 10), 1);
  } else if (typeof current === "object" && current !== null) {
    delete (current as Record<string, unknown>)[lastPart];
  }
  return clone;
}
