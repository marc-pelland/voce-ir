/**
 * Types for the inspector system.
 */

/** An IR node discovered in the compiled DOM output. */
export interface InspectedNode {
  /** The data-voce-id attribute value. */
  id: string;
  /** The DOM element this node corresponds to. */
  element: HTMLElement;
  /** Node type inferred from DOM structure. */
  type: string;
  /** Computed styles on the element. */
  styles: Record<string, string>;
  /** ARIA attributes present. */
  aria: Record<string, string>;
  /** Child node IDs. */
  children: string[];
  /** Bounding rectangle. */
  rect: DOMRect;
}

/** Current state of the inspector. */
export interface InspectorState {
  /** Whether the inspector is active. */
  active: boolean;
  /** Currently selected node ID. */
  selectedNodeId: string | null;
  /** All discovered nodes. */
  nodes: Map<string, InspectedNode>;
  /** Whether the panel is visible. */
  panelVisible: boolean;
}
